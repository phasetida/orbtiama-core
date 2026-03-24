use crate::DISPLAY_SETTINGS;

pub struct Settings {
    pub tap_speed: f32,
    pub touch_speed: f32,
    pub slide_show_offset: f32,
    pub mirror_mode: MirrorMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tap_speed: 5.0,
            touch_speed: 5.0,
            slide_show_offset: 0.0,
            mirror_mode: Default::default(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum MirrorMode {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}

pub fn get_settings<F>(f: F)
where
    F: FnMut(&mut Settings),
{
    DISPLAY_SETTINGS.with_borrow_mut(f);
}
