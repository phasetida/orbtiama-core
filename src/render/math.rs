use lyon_path::{
    geom::point,
    math::{Angle, Point},
};

use crate::deserialize::element::notes::FractureRay;

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
