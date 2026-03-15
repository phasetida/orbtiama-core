use crate::{
    CHART_STATE,
    deserialize::element::notes::{NoteDecoration, SlideBranch, SlidePart},
    render::{
        element::{
            Dense, RendHoldNote, RendSlideArrow, RendTapNote, RendTouchHoldNote, RendTouchNote,
        },
        math::{track_path, track_path_point},
    },
    state::element::{
        HoldNoteState, NoteState, SlideBranchState, SlideNoteState, TapNoteState,
        TouchHoldNoteState, TouchNoteState,
    },
};

const REND_TYPE_TAP: u8 = 1;
const REND_TYPE_TOUCH: u8 = 2;
const REND_TYPE_TOUCH_HOLD: u8 = 3;
const REND_TYPE_HOLD: u8 = 4;
const REND_TYPE_SLIDE_HEAD: u8 = 5;
const REND_TYPE_SLIDE_ARROW: u8 = 6;

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
        for it in states {
            draw_note_state(buffer, it);
        }
        buffer.write(&[0]);
    });
}

fn draw_note_state(buffer: &mut impl BufferWithCursor, state: &NoteState) {
    match state {
        NoteState::Tap(tap_note_state) => draw_tap_state(buffer, tap_note_state),
        NoteState::Touch(touch_note_state) => draw_touch_state(buffer, touch_note_state),
        NoteState::Hold(hold_note_state) => draw_hold_state(buffer, hold_note_state),
        NoteState::TouchHold(touch_hold_note_state) => {
            draw_touch_hold_state(buffer, touch_hold_note_state)
        }
        NoteState::Slide(slide_note_state) => draw_slide_state(buffer, slide_note_state),
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

fn draw_slide_state(buffer: &mut impl BufferWithCursor, state: &SlideNoteState) {
    if !state.enable {
        return;
    }
    if state.head_enable {
        buffer.write(
            RendTapNote {
                rend_type: REND_TYPE_SLIDE_HEAD,
                flags: state.note.decoration.bits(),
                x: state.x,
                y: state.y,
                rotate: state.rotate,
                scale: state.scale,
            }
            .to_bytes(),
        );
        return;
    }
    state
        .note
        .slide_branches
        .iter()
        .zip(state.branch_states.iter())
        .for_each(|(branch, state)| draw_slide_branch(buffer, branch, state));
}

fn draw_slide_branch(
    buffer: &mut impl BufferWithCursor,
    branch: &SlideBranch,
    state: &SlideBranchState,
) {
    let SlideBranchState {
        decoration,
        part_skip,
        part_move_time,
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
            let move_percent = if first {
                part_move_time / it.duration
            } else {
                0.0
            };
            draw_slide_path(buffer, it, move_percent.clamp(0.0, 1.0), &decoration, first);
        });
}

fn draw_slide_path(
    buffer: &mut impl BufferWithCursor,
    part: &SlidePart,
    percent: f32,
    decoration: &NoteDecoration,
    show_head: bool,
) {
    track_path(&part.path, percent, 25, |pos, ang| {
        buffer.write(
            RendSlideArrow {
                rend_type: REND_TYPE_SLIDE_ARROW,
                flags: decoration.bits(),
                x: pos.x,
                y: -pos.y,
                rotate: -ang.radians,
            }
            .to_bytes(),
        );
    });
    if show_head {
        let (current_point, angle, _) = track_path_point(&part.path, percent + 0.001);
        buffer.write(
            RendTapNote {
                rend_type: REND_TYPE_SLIDE_HEAD,
                flags: decoration.bits(),
                x: current_point.x,
                y: -current_point.y,
                rotate: -angle.radians,
                scale: 1.0,
            }
            .to_bytes(),
        );
    }
}
