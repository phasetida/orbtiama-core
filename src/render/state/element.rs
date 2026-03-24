use std::ops::Deref;

use lyon_path::math::{Angle, Point};

use crate::{
    deserialize::chart::element::notes::{NoteHold, NoteSlide, NoteTap, NoteTouch},
    quick_deref,
    render::{
        math::{get_fixed_position, get_fixed_rotate},
        state::setting::MirrorMode,
    },
};

pub enum NoteState {
    Tap(TapNoteState),
    Touch(TouchNoteState),
    Hold(HoldNoteState),
    TouchHold(TouchHoldNoteState),
    Slide(SlideNoteState),
}

impl Deref for NoteState {
    type Target = NoteStateCommon;

    fn deref(&self) -> &Self::Target {
        match self {
            NoteState::Tap(state) => &state.common,
            NoteState::Touch(state) => &state.common,
            NoteState::Hold(state) => &state.common,
            NoteState::TouchHold(state) => &state.common,
            NoteState::Slide(state) => &state.common,
        }
    }
}

pub struct NoteStateCommon {
    pub time: f32,
    pub enable: bool,
    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub scale: f32,
}

impl NoteStateCommon {
    pub fn set_fixed_position(&mut self, point: Point, mirror_mode: MirrorMode) {
        let point = get_fixed_position(point, mirror_mode);
        self.x = point.x;
        self.y = point.y;
    }

    pub fn set_fixed_rotate(&mut self, rotate: Angle, mirror_mode: MirrorMode) {
        self.rotate = get_fixed_rotate(rotate, mirror_mode).radians;
    }
}

impl Default for NoteStateCommon {
    fn default() -> Self {
        NoteStateCommon {
            time: 0.0,
            enable: false,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            scale: 1.0,
        }
    }
}

pub struct TapNoteState {
    pub common: NoteStateCommon,
    pub note: NoteTap,
    pub hint_scale: f32,
    pub hint_rotate: f32,
    pub hint_alpha: f32,
}

pub struct TouchNoteState {
    pub common: NoteStateCommon,
    pub progress: f32,
    pub note: NoteTouch,
}

pub struct TouchHoldNoteState {
    pub common: NoteStateCommon,
    pub show_progress: f32,
    pub hold_progress: f32,
    pub note: NoteHold,
}

pub struct HoldNoteState {
    pub common: NoteStateCommon,
    pub tail_x: f32,
    pub tail_y: f32,
    pub holding: bool,
    pub note: NoteHold,
    pub hint_scale: f32,
    pub hint_rotate: f32,
    pub hint_alpha: f32,
}

pub struct SlideNoteState {
    pub common: NoteStateCommon,
    pub head_progress: f32,
    pub body_alpha: f32,
    pub note: NoteSlide,
    pub branch_states: Vec<SlideBranchState>,
    pub hint_scale: f32,
    pub hint_rotate: f32,
    pub hint_alpha: f32,
}

#[derive(Default)]
pub struct SlideBranchState {
    pub branch_enable: bool,
    pub part_skip: usize,
    pub part_move_time: f32,
    pub part_time: f32,
}

impl HoldNoteState {
    pub fn set_fixed_tail_position(&mut self, point: Point, mirror_mode: MirrorMode) {
        let point = get_fixed_position(point, mirror_mode);
        self.tail_x = point.x;
        self.tail_y = point.y;
    }
}

impl TapNoteState {
    pub fn set_fixed_hint_rotate(&mut self, mirror_mode: MirrorMode) {
        self.hint_rotate = get_fixed_rotate(
            self.note
                .location
                .get_point()
                .to_vector()
                .angle_from_x_axis(),
            mirror_mode,
        )
        .radians;
    }
}

impl HoldNoteState {
    pub fn set_fixed_hint_rotate(&mut self, mirror_mode: MirrorMode) {
        self.hint_rotate = get_fixed_rotate(
            self.note
                .location
                .get_point()
                .to_vector()
                .angle_from_x_axis(),
            mirror_mode,
        )
        .radians;
    }
}

impl SlideNoteState {
    pub fn set_fixed_hint_rotate(&mut self, mirror_mode: MirrorMode) {
        self.hint_rotate = get_fixed_rotate(
            self.note
                .location
                .get_point()
                .to_vector()
                .angle_from_x_axis(),
            mirror_mode,
        )
        .radians;
    }
}

quick_deref!(TapNoteState, NoteStateCommon, common);
quick_deref!(TouchNoteState, NoteStateCommon, common);
quick_deref!(HoldNoteState, NoteStateCommon, common);
quick_deref!(TouchHoldNoteState, NoteStateCommon, common);
quick_deref!(SlideNoteState, NoteStateCommon, common);
