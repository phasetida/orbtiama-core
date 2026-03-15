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
    if progress <= 0.2 {
        (start_point, progress / 0.2)
    } else {
        let move_progress = (progress - 0.2) / 0.8;
        let delta = (end_point - start_point) * move_progress;
        (start_point + delta, 1.0)
    }
}
