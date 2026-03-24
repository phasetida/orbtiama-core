use std::f32::consts::PI;

use lyon_path::math::Angle;

use crate::{
    CHART_STATE, DISPLAY_SETTINGS,
    deserialize::chart::element::notes::SlideBranch,
    render::{
        math::{get_slide_alpha, get_tap_position_and_scale},
        state::{
            element::{
                HoldNoteState, NoteState, SlideBranchState, SlideNoteState, TapNoteState,
                TouchHoldNoteState, TouchNoteState,
            },
            setting::Settings,
        },
    },
};

pub fn tick_state(time_in_second: f32, delta_time_in_second: f32) {
    CHART_STATE.with_borrow_mut(|states| {
        DISPLAY_SETTINGS.with_borrow(|setting| {
            tick_notes(time_in_second, delta_time_in_second, states, setting);
        })
    })
}

fn tick_notes(
    time_in_second: f32,
    _delta_time_in_second: f32,
    states: &mut [NoteState],
    setting: &Settings,
) {
    for it in states {
        tick_note(time_in_second, it, setting);
    }
}

fn tick_note(time_in_second: f32, note: &mut NoteState, setting: &Settings) {
    match note {
        NoteState::Tap(tap_note_state) => tick_tap_note(time_in_second, tap_note_state, setting),
        NoteState::Touch(touch_note_state) => {
            tick_touch_note(time_in_second, touch_note_state, setting)
        }
        NoteState::Hold(hold_note_state) => {
            tick_hold_note(time_in_second, hold_note_state, setting)
        }
        NoteState::TouchHold(touch_hold_note_state) => {
            tick_touch_hold_note(time_in_second, touch_hold_note_state, setting)
        }
        NoteState::Slide(slide_note_state) => {
            tick_slide_note(time_in_second, slide_note_state, setting)
        }
    }
}

fn get_progress(speed: f32, current_time: f32, note_time: f32) -> f32 {
    let approach_time = 3.6 / speed;
    1.0 - (note_time - current_time) / approach_time
}

fn tick_tap_note(time_in_second: f32, state: &mut TapNoteState, setting: &Settings) {
    let speed = setting.tap_speed;
    let progress = get_progress(speed, time_in_second, state.time);
    if progress <= 0.0 || progress >= 1.0 {
        state.common.enable = false;
        return;
    }
    state.common.enable = true;
    let (position, scale) = get_tap_position_and_scale(&state.note.location, progress);
    state
        .common
        .set_fixed_position(position, setting.mirror_mode);
    state.common.scale = scale;
    state.common.set_fixed_rotate(
        position.to_vector().angle_from_x_axis(),
        setting.mirror_mode,
    );
    state.set_fixed_hint_rotate(setting.mirror_mode);
    state.hint_scale = (((progress - 0.5) / 0.5 * 0.8) + 0.2).clamp(0.2, 1.0);
    state.hint_alpha = scale;
}

fn tick_touch_note(time_in_second: f32, state: &mut TouchNoteState, setting: &Settings) {
    let speed = setting.touch_speed;
    let progress = get_progress(speed, time_in_second, state.time);
    if progress <= 0.0 || progress >= 1.0 {
        state.common.enable = false;
        return;
    }
    state.common.enable = true;
    let point = state.note.location.get_point();
    state.common.set_fixed_position(point, setting.mirror_mode);
    state.progress = progress;
}

fn tick_hold_note(time_in_second: f32, state: &mut HoldNoteState, setting: &Settings) {
    let speed = setting.tap_speed;
    let head_progress = get_progress(speed, time_in_second, state.time);
    let tail_progress = get_progress(speed, time_in_second, state.time + state.note.duration);
    if head_progress <= 0.0 || tail_progress >= 1.0 {
        state.common.enable = false;
        return;
    }
    state.common.enable = true;
    let (head_position, head_scale) =
        get_tap_position_and_scale(&state.note.location, head_progress.clamp(0.0, 1.0));
    let (tail_position, _) =
        get_tap_position_and_scale(&state.note.location, tail_progress.clamp(0.0, 1.0));
    state
        .common
        .set_fixed_position(head_position, setting.mirror_mode);
    state.set_fixed_tail_position(tail_position, setting.mirror_mode);
    state.common.scale = head_scale;
    state.common.set_fixed_rotate(
        head_position.to_vector().angle_from_x_axis(),
        setting.mirror_mode,
    );
    state.holding = head_progress >= 1.0;
    state.set_fixed_hint_rotate(setting.mirror_mode);
    state.hint_scale = (((head_progress - 0.5) / 0.5 * 0.8) + 0.2).clamp(0.2, 1.0);
    state.hint_alpha = head_scale;
}

fn tick_touch_hold_note(time_in_second: f32, state: &mut TouchHoldNoteState, setting: &Settings) {
    let speed = setting.touch_speed;
    let show_progress = get_progress(speed, time_in_second, state.time);
    let hold_progress =
        1.0 - (state.time + state.note.duration - time_in_second) / state.note.duration;
    if show_progress <= 0.0 || hold_progress >= 1.0 {
        state.common.enable = false;
        return;
    }
    let position = state.note.location.get_point();
    state.common.enable = true;
    state
        .common
        .set_fixed_position(position, setting.mirror_mode);
    state.show_progress = show_progress.clamp(0.0, 1.0);
    state.hold_progress = hold_progress.clamp(0.0, 1.0);
}

fn tick_slide_note(time_in_second: f32, state: &mut SlideNoteState, setting: &Settings) {
    let speed = setting.tap_speed;
    let note_time = state.time;
    let head_progress = get_progress(speed, time_in_second, note_time);
    if head_progress <= 0.0 {
        state.common.enable = false;
        return;
    }
    let slide_alpha_offset = setting.slide_show_offset;
    state.common.enable = true;
    state.head_progress = head_progress;
    state.body_alpha = get_slide_alpha(slide_alpha_offset, head_progress);
    state.hint_scale = 0.0;
    if head_progress <= 1.0 {
        let (position, scale) =
            get_tap_position_and_scale(&state.note.location, head_progress.clamp(0.0, 1.0));
        state
            .common
            .set_fixed_position(position, setting.mirror_mode);
        state.common.scale = scale;
        state.common.set_fixed_rotate(
            position.to_vector().angle_from_x_axis()
                + Angle {
                    radians: (scale * 2.0 * PI),
                },
            setting.mirror_mode,
        );
        state.set_fixed_hint_rotate(setting.mirror_mode);
        state.hint_scale = (((head_progress - 0.5) / 0.5 * 0.8) + 0.2).clamp(0.2, 1.0);
        state.hint_alpha = scale;
    }
    state
        .note
        .slide_branches
        .iter_mut()
        .enumerate()
        .for_each(|(i, it)| {
            tick_slide_branch(time_in_second, note_time, it, &mut state.branch_states[i]);
        });
}

fn tick_slide_branch(
    time_in_second: f32,
    start_time: f32,
    branch: &SlideBranch,
    branch_state: &mut SlideBranchState,
) {
    let time_relative = time_in_second - start_time;
    let Some((time_part_start, skip, current_part)) = branch
        .slide_parts
        .iter()
        .enumerate()
        .try_fold(0.0, |time_past, (index, it)| {
            let part_duration = it.delay_duration + it.duration;
            if time_past + part_duration >= time_relative {
                Err((time_past, index, it))
            } else {
                Ok(time_past + part_duration)
            }
        })
        .err()
    else {
        branch_state.branch_enable = false;
        return;
    };
    branch_state.branch_enable = true;
    let part_time = time_relative - time_part_start;
    let part_move_time = part_time - current_part.delay_duration;
    branch_state.part_skip = skip;
    branch_state.part_move_time = part_move_time;
    branch_state.part_time = part_time;
}
