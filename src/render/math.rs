use std::f32::consts::PI;

use lyon_path::math::{Angle, Point, point};

use crate::{
    deserialize::chart::element::notes::{FractureRay, Location},
    render::state::setting::MirrorMode,
};

pub fn track_path_point(path: &FractureRay, percent: f32) -> (Point, Angle, usize) {
    let target_length = path.length * percent;
    path.path
        .iter()
        .enumerate()
        .try_fold(0.0, |length, (i, it)| match it {
            lyon_path::Event::Line { from, to } => {
                let fracture_length = from.distance_to(to);
                if length + fracture_length >= target_length {
                    Err((to, (to - from).angle_from_x_axis(), i))
                } else {
                    Ok(length + fracture_length)
                }
            }
            _ => Ok(length),
        })
        .err()
        .unwrap_or((point(-9.9, -9.9), Angle::zero(), 0))
}

pub fn track_path<F>(path: &FractureRay, percent: f32, step: usize, mut func: F)
where
    F: FnMut(Point, Angle),
{
    let (_, _, skip) = track_path_point(path, percent);
    path.path
        .iter()
        .step_by(step)
        .skip(skip / step)
        .for_each(|it| {
            if let lyon_path::Event::Line { from, to } = it {
                func(from, (to - from).angle_from_x_axis());
            }
        });
}

pub fn track_path_complete<F>(path: &FractureRay, length_per_step: f32, mut func: F)
where
    F: FnMut(Point, Angle, f32),
{
    let length = path.length;
    let steps = length / length_per_step;
    path.path
        .iter()
        .step_by((length_per_step / path.tolerance) as usize)
        .enumerate()
        .for_each(|(i, it)| {
            if let lyon_path::Event::Line { from, to } = it {
                func(
                    from,
                    (to - from).angle_from_x_axis(),
                    ((i as f32) / steps).clamp(0.0, 1.0),
                )
            }
        });
}

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
    let cs = 0.5 - 0.5 * offset;
    let k = 1.0 / cs;
    if head_progress < c {
        0.0
    } else {
        k * (head_progress - c)
    }
}

pub fn get_fixed_rotate(rotate: Angle, mirror_mode: MirrorMode) -> Angle {
    let rotate = rotate.radians;
    Angle {
        radians: match mirror_mode {
            MirrorMode::None => -rotate,
            MirrorMode::Horizontal => rotate + PI,
            MirrorMode::Vertical => rotate,
            MirrorMode::Both => PI - rotate,
        },
    }
}

pub fn get_fixed_position(point: Point, mirror_mode: MirrorMode) -> Point {
    let x_sign = match mirror_mode {
        MirrorMode::None | MirrorMode::Vertical => 1.0,
        MirrorMode::Horizontal | MirrorMode::Both => -1.0,
    };
    let y_sign = match mirror_mode {
        MirrorMode::None | MirrorMode::Horizontal => -1.0,
        MirrorMode::Vertical | MirrorMode::Both => 1.0,
    };
    lyon_path::math::point(point.x * x_sign, point.y * y_sign)
}
