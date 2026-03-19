#[repr(C, packed)]
pub struct RendTapNote {
    pub rend_type: u8,
    pub flags: u16,
    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub scale: f32,
    pub alpha: f32,
}

#[repr(C, packed)]
pub struct RendTouchNote {
    pub rend_type: u8,
    pub flags: u16,
    pub x: f32,
    pub y: f32,
    pub progress: f32,
}

#[repr(C, packed)]
pub struct RendTouchHoldNote {
    pub rend_type: u8,
    pub flags: u16,
    pub x: f32,
    pub y: f32,
    pub show_progress: f32,
    pub hold_progress: f32,
}

#[repr(C, packed)]
pub struct RendHoldNote {
    pub rend_type: u8,
    pub flags: u16,
    pub head_x: f32,
    pub head_y: f32,
    pub tail_x: f32,
    pub tail_y: f32,
    pub rotate: f32,
    pub scale: f32,
    pub holding: bool,
}

#[repr(C, packed)]
pub struct RendSlideArrow {
    pub rend_type: u8,
    pub flags: u16,
    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub alpha: f32,
}

#[repr(C, packed)]
pub struct RendHintLine {
    pub rend_type: u8,
    pub flags: u16,
    pub slide: bool,
    pub scale: f32,
    pub rotate: f32,
    pub alpha: f32,
}

#[repr(C, packed)]
pub struct RendEachHintLine {
    pub rend_type: u8,
    pub serial: u8,
    pub scale: f32,
    pub rotate: f32,
}

#[repr(C, packed)]
pub struct RendSlideWifiArrow {
    pub rend_type: u8,
    pub flags: u16,
    pub index: u8,
    pub x: f32,
    pub y: f32,
    pub rotate: f32,
    pub alpha: f32,
}

impl Dense for RendTapNote {}
impl Dense for RendTouchNote {}
impl Dense for RendHoldNote {}
impl Dense for RendTouchHoldNote {}
impl Dense for RendSlideArrow {}
impl Dense for RendHintLine {}
impl Dense for RendEachHintLine {}
impl Dense for RendSlideWifiArrow {}

pub trait Dense {
    fn to_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<Self>(self).cast::<u8>(),
                std::mem::size_of::<Self>(),
            )
        }
    }
}
