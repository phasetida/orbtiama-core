use std::ops::Deref;

use lyon_path::math::Point;

use crate::deserialize::element::notes::{NoteHold, NoteSlide, NoteTap, NoteTouch};

pub enum NoteState {
    Tap(TapNoteState),
    Touch(TouchNoteState),
    Hold(HoldNoteState),
    TouchHold(TouchHoldNoteState),
    Slide(SlideNoteState),
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
    pub fn set_fixed_position(&mut self, point: Point) {
        self.x = point.x;
        self.y = -point.y;
    }

    pub fn set_fixed_rotate(&mut self, rotate: f32) {
        self.rotate = -rotate;
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

impl HoldNoteState {
    pub fn set_fixed_tail_position(&mut self, point: Point) {
        self.tail_x = point.x;
        self.tail_y = -point.y;
    }
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

impl Deref for TapNoteState {
    type Target = NoteStateCommon;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for TouchNoteState {
    type Target = NoteStateCommon;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for HoldNoteState {
    type Target = NoteStateCommon;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for TouchHoldNoteState {
    type Target = NoteStateCommon;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for SlideNoteState {
    type Target = NoteStateCommon;
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
