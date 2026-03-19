use lyon_path::math::Point;

use crate::deserialize::element::notes::Location;

impl Location {
    pub fn get_tap_position_and_scale(&self, progress: f32) -> (Point, f32) {
        get_tap_position_and_scale(self, progress)
    }
}

pub fn get_tap_position_and_scale(location: &Location, progress: f32) -> (Point, f32) {
    let end_point = location.get_point();
    let start_point = end_point * 0.2;
    let division = 0.5;
    if progress <= division {
        (start_point, progress / division)
    } else {
        let move_progress = (progress - division) / (1.0 - division);
        let delta = (end_point - start_point) * move_progress;
        (start_point + delta, 1.0)
    }
}

pub fn get_slide_alpha(offset: f32, head_progress: f32) -> f32 {
    let offset = offset.clamp(-1.0, 1.0);
    let head_progress = head_progress.clamp(0.0, 1.0);
    if offset >= 0.9999 {
        return if head_progress == 1.0 { 1.0 } else { 0.0 };
    }
    let c = 0.5 + 0.5 * offset;
    let cs = 1.0 - c;
    let k = 1.0 / cs;
    if head_progress < cs {
        0.0
    } else {
        k * (head_progress - c)
    }
}
