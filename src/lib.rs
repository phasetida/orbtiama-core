use std::cell::RefCell;

use crate::render::state::{element::NoteState, setting::Settings};

pub mod deserialize;
pub mod render;
pub mod statistics;
mod util;

thread_local! {
    pub(crate) static CHART_STATE: RefCell<Vec<NoteState>> = RefCell::new(Vec::default());
    pub(crate) static DISPLAY_SETTINGS: RefCell<Settings> = RefCell::new(Settings::default());
}
