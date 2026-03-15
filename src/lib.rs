use std::cell::RefCell;

use crate::state::{element::NoteState, setting::Settings};

pub mod deserialize;
pub mod render;
pub mod state;
mod test;

thread_local! {
    pub(crate) static CHART_STATE: RefCell<Vec<NoteState>> = RefCell::new(Default::default());
    pub(crate) static DISPLAY_SETTINGS: RefCell<Settings> = RefCell::new(Default::default());
}
