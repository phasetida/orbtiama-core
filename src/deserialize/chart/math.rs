use lyon_path::{
    Path,
    geom::euclid::point2,
    math::{Angle, Point, vector},
    traits::PathIterator,
};

use crate::deserialize::chart::element::notes::{FractureRay, Location};

#[derive(Debug, Clone, Copy)]
pub enum Clockwise {
    Cw,
    Ccw,
}

impl Clockwise {
    pub fn reversed(self) -> Self {
        (!bool::from(self)).into()
    }
}

impl From<Clockwise> for bool {
    fn from(value: Clockwise) -> Self {
        match value {
            Clockwise::Cw => true,
            Clockwise::Ccw => false,
        }
    }
}

impl From<bool> for Clockwise {
    fn from(value: bool) -> Self {
        match value {
            true => Clockwise::Cw,
            false => Clockwise::Ccw,
        }
    }
}

pub fn get_position_in_circle(radius: f32, rad: Angle) -> Point {
    let (sin, cos) = rad.sin_cos();
    point2(radius * cos, radius * sin)
}

pub fn get_circular_distance_with_direction(
    a: u8,
    b: u8,
    total_length: u8,
    clockwise: Clockwise,
) -> u8 {
    if a < 1 || a > total_length || b < 1 || b > total_length {
        return 0;
    }
    if a == b {
        return total_length;
    }
    let a = a.cast_signed();
    let b = b.cast_signed();
    let w = total_length.cast_signed();
    match clockwise {
        Clockwise::Cw => ((b - a + w) % w).cast_unsigned(),
        Clockwise::Ccw => ((a - b + w) % w).cast_unsigned(),
    }
}

pub fn get_nearest_circular_distance(a: u8, b: u8, total_length: u8) -> (u8, Clockwise) {
    if a < 1 || a > total_length || b < 1 || b > total_length {
        return (0, Clockwise::Cw);
    }
    let t = (b.cast_signed() - a.cast_signed()) % (total_length.cast_signed());
    let t = if t < 0 {
        t + total_length.cast_signed()
    } else {
        t
    };
    let cw_steps = t;
    let ccw_steps = total_length.cast_signed() - t;
    if t == 0 {
        (0, Clockwise::Cw)
    } else if cw_steps < ccw_steps {
        (cw_steps.cast_unsigned(), Clockwise::Cw)
    } else if ccw_steps < cw_steps {
        (ccw_steps.cast_unsigned(), Clockwise::Ccw)
    } else {
        (cw_steps.cast_unsigned(), Clockwise::Cw)
    }
}

pub fn move_circular(a: u8, b: u8, total_length: u8, clockwise: Clockwise) -> u8 {
    let a = a.cast_signed();
    let b = b.cast_signed();
    let total_length = total_length.cast_signed();
    if clockwise.into() {
        (a + b - 1) % total_length + 1
    } else {
        (a - b + (total_length - 1)) % total_length + 1
    }
    .cast_unsigned()
}

pub fn rotate_point(center: Point, point_a: Point, theta: Angle) -> Point {
    let d = point_a - center;
    let (sin, cos) = theta.sin_cos();
    let dx_rotated = d.x * cos - d.y * sin;
    let dy_rotated = d.x * sin + d.y * cos;
    center + vector(dx_rotated, dy_rotated)
}

pub fn determine_clockwise(location: &Location, right: bool) -> Clockwise {
    let right_clockwise: [u8; 4] = [7, 8, 1, 2];
    if right {
        right_clockwise.contains(&location.index).into()
    } else {
        (!right_clockwise.contains(&location.index)).into()
    }
}

pub fn linearize_path(path: Path) -> FractureRay {
    let mut builder = Path::builder();
    builder.begin(
        path.as_slice()
            .iter()
            .next()
            .expect("internal failure: empty path")
            .from(),
    );
    let tolerance = 0.002;
    for it in path.iter().flattened(tolerance) {
        match it {
            lyon_path::Event::Begin { at } => {
                builder.line_to(at);
            }
            lyon_path::Event::Line { from, to } => {
                let vector = to - from;
                let length = vector.length();
                let vector = vector.with_length(tolerance);
                (1..(length / tolerance) as i32).for_each(|it| {
                    builder.line_to(from + vector * (it as f32));
                });
                builder.line_to(to);
            }
            _ => {}
        };
    }
    builder.end(false);
    let path = builder.build();
    let length = path.iter().fold(0.0, |i, it| {
        i + match it {
            lyon_path::Event::Line { from, to } => from.distance_to(to),
            _ => 0.0,
        }
    });
    FractureRay {
        path,
        length,
        tolerance,
    }
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;
    use lyon_path::math::{Angle, point};

    use crate::{
        deserialize::chart::math::{
            Clockwise, get_circular_distance_with_direction, get_nearest_circular_distance,
            rotate_point,
        },
        function_test,
    };

    function_test!(gcdad_test_1, get_nearest_circular_distance, 1, 8, 8);
    function_test!(gcdad_test_2, get_nearest_circular_distance, 8, 1, 8);
    function_test!(gcdad_test_3, get_nearest_circular_distance, 3, 8, 8);
    function_test!(gcdad_test_4, get_nearest_circular_distance, 8, 3, 8);
    function_test!(gcdad_test_5, get_nearest_circular_distance, 8, 4, 8);
    function_test!(
        gcd_test_1,
        get_circular_distance_with_direction,
        8,
        4,
        8,
        Clockwise::Cw
    );
    function_test!(
        gcd_test_2,
        get_circular_distance_with_direction,
        1,
        8,
        8,
        Clockwise::Cw
    );
    function_test!(
        gcd_test_3,
        get_circular_distance_with_direction,
        8,
        1,
        8,
        Clockwise::Ccw
    );
    function_test!(
        gcd_test_4,
        get_circular_distance_with_direction,
        1,
        1,
        8,
        Clockwise::Ccw
    );
    function_test!(
        gcd_test_5,
        get_circular_distance_with_direction,
        4,
        5,
        8,
        Clockwise::Ccw
    );
    function_test!(
        rp_test_1,
        rotate_point,
        point(0.0, 0.0),
        point(1.0, 0.0),
        Angle::pi()
    );
}
