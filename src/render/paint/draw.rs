use crate::{
    CHART_STATE, DISPLAY_SETTINGS,
    deserialize::chart::element::notes::{NoteDecoration, SlideBranch, SlidePart},
    render::{
        math::{
            get_fixed_position, get_fixed_rotate, track_path, track_path_complete, track_path_point,
        },
        paint::element::{
            RendHintLine, RendHoldNote, RendSlideArrow, RendSlideWifiArrow, RendTapNote,
            RendTouchHoldNote, RendTouchNote,
        },
        state::{
            element::{
                HoldNoteState, NoteState, SlideBranchState, SlideNoteState, TapNoteState,
                TouchHoldNoteState, TouchNoteState,
            },
            setting::MirrorMode,
        },
    },
    util::Dense,
};

const REND_TYPE_TAP: u8 = 1;
const REND_TYPE_TOUCH: u8 = 2;
const REND_TYPE_TOUCH_HOLD: u8 = 3;
const REND_TYPE_HOLD: u8 = 4;
const REND_TYPE_SLIDE_HEAD: u8 = 5;
const REND_TYPE_SLIDE_ARROW: u8 = 6;
const REND_TYPE_HINT: u8 = 7;
// const REND_TYPE_EACH_HINT: u8 = 8;
const REND_TYPE_WIFI_PART: u8 = 9;

/// A trait for observing write operations on a buffer.
///
/// The caller needs to implement this trait to listen for write events on the
/// buffer, which typically contains a cursor.
pub trait BufferWithCursor {
    /// Called when `slice` is written to the buffer.
    fn write(&mut self, slice: &[u8]);
}

pub fn draw(buffer: &mut impl BufferWithCursor) {
    CHART_STATE.with_borrow(|states| {
        DISPLAY_SETTINGS.with_borrow(|settings| {
            for it in states.iter().rev() {
                draw_note_state(buffer, it, settings.mirror_mode);
            }
            buffer.write(&[0]);
        });
    });
}

fn draw_note_state(buffer: &mut impl BufferWithCursor, state: &NoteState, mirror_mode: MirrorMode) {
    match state {
        NoteState::Tap(tap_note_state) => draw_tap_state(buffer, tap_note_state),
        NoteState::Touch(touch_note_state) => draw_touch_state(buffer, touch_note_state),
        NoteState::Hold(hold_note_state) => draw_hold_state(buffer, hold_note_state),
        NoteState::TouchHold(touch_hold_note_state) => {
            draw_touch_hold_state(buffer, touch_hold_note_state)
        }
        NoteState::Slide(slide_note_state) => {
            draw_slide_state(buffer, slide_note_state, mirror_mode)
        }
    }
}

fn draw_tap_state(buffer: &mut impl BufferWithCursor, state: &TapNoteState) {
    if !state.enable {
        return;
    }
    buffer.write(
        RendTapNote {
            rend_type: REND_TYPE_TAP,
            flags: state.note.decoration.bits(),
            x: state.x,
            y: state.y,
            rotate: state.rotate,
            scale: state.scale,
            alpha: 1.0,
        }
        .to_bytes(),
    );
    buffer.write(
        RendHintLine {
            rend_type: REND_TYPE_HINT,
            flags: state.note.decoration.bits(),
            slide: false,
            scale: state.hint_scale,
            rotate: state.hint_rotate,
            alpha: state.hint_alpha,
        }
        .to_bytes(),
    );
}

fn draw_touch_state(buffer: &mut impl BufferWithCursor, state: &TouchNoteState) {
    if !state.enable {
        return;
    }
    buffer.write(
        RendTouchNote {
            rend_type: REND_TYPE_TOUCH,
            flags: state.note.decoration.bits(),
            x: state.x,
            y: state.y,
            progress: state.progress,
        }
        .to_bytes(),
    );
}

fn draw_hold_state(buffer: &mut impl BufferWithCursor, state: &HoldNoteState) {
    if !state.enable {
        return;
    }
    buffer.write(
        RendHoldNote {
            rend_type: REND_TYPE_HOLD,
            flags: state.note.decoration.bits(),
            head_x: state.x,
            head_y: state.y,
            tail_x: state.tail_x,
            tail_y: state.tail_y,
            rotate: state.rotate,
            scale: state.scale,
            holding: state.holding,
        }
        .to_bytes(),
    );
    buffer.write(
        RendHintLine {
            rend_type: REND_TYPE_HINT,
            flags: state.note.decoration.bits(),
            slide: false,
            scale: state.hint_scale,
            rotate: state.hint_rotate,
            alpha: state.hint_alpha,
        }
        .to_bytes(),
    );
}

fn draw_touch_hold_state(buffer: &mut impl BufferWithCursor, state: &TouchHoldNoteState) {
    if !state.enable {
        return;
    }
    buffer.write(
        RendTouchHoldNote {
            rend_type: REND_TYPE_TOUCH_HOLD,
            flags: state.note.decoration.bits(),
            x: state.x,
            y: state.y,
            show_progress: state.show_progress,
            hold_progress: state.hold_progress,
        }
        .to_bytes(),
    );
}

fn draw_slide_state(
    buffer: &mut impl BufferWithCursor,
    state: &SlideNoteState,
    mirror_mode: MirrorMode,
) {
    if !state.enable {
        return;
    }
    if state.head_progress <= 1.0 {
        buffer.write(
            RendTapNote {
                rend_type: REND_TYPE_SLIDE_HEAD,
                flags: state.note.decoration.bits(),
                x: state.x,
                y: state.y,
                rotate: state.rotate,
                scale: state.scale,
                alpha: 1.0,
            }
            .to_bytes(),
        );
        buffer.write(
            RendHintLine {
                rend_type: REND_TYPE_HINT,
                flags: state.note.decoration.bits(),
                slide: true,
                scale: state.hint_scale,
                rotate: state.hint_rotate,
                alpha: state.hint_alpha,
            }
            .to_bytes(),
        );
    }
    state
        .note
        .slide_branches
        .iter()
        .zip(state.branch_states.iter())
        .for_each(|(branch, branch_state)| {
            if state.body_alpha <= 0.0 {
                return;
            }
            draw_slide_branch(buffer, branch, branch_state, state.body_alpha, mirror_mode);
        });
}

fn draw_slide_branch(
    buffer: &mut impl BufferWithCursor,
    branch: &SlideBranch,
    state: &SlideBranchState,
    alpha: f32,
    mirror_mode: MirrorMode,
) {
    let SlideBranchState {
        part_skip,
        part_move_time,
        part_time,
        branch_enable,
    } = *state;
    if !branch_enable {
        return;
    }
    branch
        .slide_parts
        .iter()
        .skip(part_skip)
        .enumerate()
        .for_each(|(i, it)| {
            let first = i == 0;
            let (move_percent, delay_percent) = if first {
                (part_move_time / it.duration, part_time / it.delay_duration)
            } else {
                (0.0, 0.0)
            };
            draw_slide_path(
                buffer,
                it,
                move_percent.clamp(0.0, 1.0),
                &it.decoration,
                delay_percent,
                alpha,
                mirror_mode,
            );
        });
}

fn draw_slide_path(
    buffer: &mut impl BufferWithCursor,
    part: &SlidePart,
    percent: f32,
    part_decoration: &NoteDecoration,
    tracking_head_percent: f32,
    alpha: f32,
    mirror_mode: MirrorMode,
) {
    track_path(&part.path, percent, 25, |pos, ang| {
        let pos = get_fixed_position(pos, mirror_mode);
        let ang = get_fixed_rotate(ang, mirror_mode);
        buffer.write(
            RendSlideArrow {
                rend_type: REND_TYPE_SLIDE_ARROW,
                flags: part_decoration.bits(),
                x: pos.x,
                y: pos.y,
                rotate: ang.radians,
                alpha,
            }
            .to_bytes(),
        );
    });
    if part_decoration.contains(NoteDecoration::WifiSlide) {
        track_path_complete(&part.path, 0.085, |pos, ang, i| {
            let pos = get_fixed_position(pos, mirror_mode);
            let ang = get_fixed_rotate(ang, mirror_mode);
            buffer.write(
                RendSlideWifiArrow {
                    rend_type: REND_TYPE_WIFI_PART,
                    flags: part_decoration.bits(),
                    index: (i * 11.0) as u8,
                    x: pos.x,
                    y: pos.y,
                    rotate: ang.radians,
                    alpha: if i < percent { 0.0 } else { alpha },
                }
                .to_bytes(),
            );
        });
    }
    let show_tracking_head = tracking_head_percent > 0.0;
    if show_tracking_head {
        let (current_point, angle, _) = track_path_point(&part.path, percent + 0.001);
        let current_point = get_fixed_position(current_point, mirror_mode);
        let angle = get_fixed_rotate(angle, mirror_mode);
        buffer.write(
            RendTapNote {
                rend_type: REND_TYPE_SLIDE_HEAD,
                flags: part_decoration.bits(),
                x: current_point.x,
                y: current_point.y,
                rotate: angle.radians,
                scale: tracking_head_percent.clamp(0.0, 1.0),
                alpha: tracking_head_percent.clamp(0.0, 1.0),
            }
            .to_bytes(),
        );
    }
}
