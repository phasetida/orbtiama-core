pub struct Settings {
    pub tap_speed: f32,
    pub touch_speed: f32,
    pub slide_show_offset: f32,
    pub mirror_mode: MirrorMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tap_speed: 8.0,
            touch_speed: 8.0,
            slide_show_offset: 0.0,
            mirror_mode: Default::default(),
        }
    }
}

#[derive(Default)]
pub enum MirrorMode {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}
