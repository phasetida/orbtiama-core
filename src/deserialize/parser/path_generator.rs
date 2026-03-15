use std::f32::consts::PI;

use lyon_path::{
    ArcFlags, Path,
    math::{Angle, Point, point, vector},
    traits::SvgPathBuilder,
};

use crate::deserialize::{
    element::notes::{Location, SlideLocator},
    math::{Clockwise, determine_clockwise, get_position_in_circle, rotate_point},
};

pub fn generate_path(from_location: Location, slide_locator: SlideLocator) -> Path {
    let mut builder = Path::builder().with_svg();
    let start_pos = from_location.get_point();
    builder.move_to(start_pos);
    match slide_locator {
        SlideLocator::Invalid => panic!("path_generator: invalid path"),
        SlideLocator::Straight(to_location) => generate_straight(&mut builder, to_location),
        SlideLocator::ClockwiseArc(to_location) => generate_side_arc(
            &mut builder,
            from_location,
            to_location,
            determine_clockwise(&from_location, true),
        ),
        SlideLocator::AntiClockwiseArc(to_location) => generate_side_arc(
            &mut builder,
            from_location,
            to_location,
            determine_clockwise(&from_location, false),
        ),
        SlideLocator::ShortArc(to_location) => {
            let (_, clockwise) = from_location.tap_distance_to(&to_location);
            generate_side_arc(&mut builder, from_location, to_location, clockwise)
        }
        SlideLocator::VShape(to_location) => generate_v(&mut builder, to_location),
        SlideLocator::GrandVShape(to_location_1, to_location_2) => {
            generator_grand_v(&mut builder, to_location_1, to_location_2)
        }
        SlideLocator::ThunderboltS(to_location) => {
            generate_zigzag(&mut builder, from_location, to_location, false)
        }
        SlideLocator::ThunderboltZ(to_location) => {
            generate_zigzag(&mut builder, from_location, to_location, true)
        }
        SlideLocator::PShape(to_location) => {
            generate_q_p_curve(&mut builder, from_location, to_location, Clockwise::Ccw);
        }
        SlideLocator::QShape(to_location) => {
            generate_q_p_curve(&mut builder, from_location, to_location, Clockwise::Cw)
        }
        SlideLocator::GrandPShape(to_location) => {
            generate_qq_pp_curve(&mut builder, from_location, to_location, Clockwise::Ccw)
        }
        SlideLocator::GrandQShape(to_location) => {
            generate_qq_pp_curve(&mut builder, from_location, to_location, Clockwise::Cw);
        }
        SlideLocator::FanShape(to_location) => generate_fan_line(&mut builder, to_location),
    };
    builder.build()
}

fn generate_straight(builder: &mut impl SvgPathBuilder, to_location: Location) {
    builder.line_to(to_location.get_point());
}

fn generate_side_arc(
    builder: &mut impl SvgPathBuilder,
    from_location: Location,
    to_location: Location,
    clockwise: Clockwise,
) {
    let unit_distance = from_location.tap_distance_to_with_clockwise(&to_location, clockwise);
    builder.arc_to(
        vector(0.5, 0.5),
        Angle::zero(),
        ArcFlags {
            large_arc: unit_distance >= 4,
            sweep: clockwise.reversed().into(),
        },
        to_location.get_point(),
    );
}

fn generate_v(builder: &mut impl SvgPathBuilder, to_location: Location) {
    builder.line_to(point(0.0, 0.0));
    builder.line_to(to_location.get_point());
}

fn generator_grand_v(
    builder: &mut impl SvgPathBuilder,
    to_location_1: Location,
    to_location_2: Location,
) {
    builder.line_to(to_location_1.get_point());
    builder.line_to(to_location_2.get_point());
}

/// 什么？你说你看不懂这段？
/// 不错，nofyso也看不懂的说，因为都是抄的
fn generate_zigzag(
    builder: &mut impl SvgPathBuilder,
    from_location: Location,
    to_location: Location,
    is_z: bool,
) {
    let start = from_location.get_point();
    let end = to_location.get_point();
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 {
        builder.line_to(point(end.x, end.y));
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let nx = -uy;
    let ny = ux;
    let offset_amount = 0.2;
    let sign = if is_z { -1.0 } else { 1.0 };
    let p1 = point(
        start.x + ux * (len * 0.49) + nx * offset_amount * sign,
        start.y + uy * (len * 0.49) + ny * offset_amount * sign,
    );
    let p2 = point(
        start.x + ux * (len * 0.51) - nx * offset_amount * sign,
        start.y + uy * (len * 0.51) - ny * offset_amount * sign,
    );
    builder.line_to(point(p1.x, p1.y));
    builder.line_to(point(p2.x, p2.y));
    builder.line_to(point(end.x, end.y));
}

fn generate_q_p_curve(
    builder: &mut impl SvgPathBuilder,
    from_location: Location,
    to_location: Location,
    clockwise: Clockwise,
) {
    let r_mid = (PI / 8.0).sin() * 0.5;
    let alpha = Angle {
        radians: 67.5f32.to_radians() + 0.01,
    };
    let (distance, _) = from_location.tap_distance_to(&to_location);
    let in_angle = from_location.get_point().to_vector().angle_from_x_axis();
    let out_angle = to_location.get_point().to_vector().angle_from_x_axis();
    let in_point = get_position_in_circle(
        r_mid,
        in_angle - if clockwise.into() { alpha } else { -alpha },
    );
    let out_point = get_position_in_circle(
        r_mid,
        out_angle + if clockwise.into() { alpha } else { -alpha },
    );
    let radii = vector(r_mid, r_mid);
    let rev_distance = from_location.tap_distance_to_with_clockwise(&to_location, clockwise);
    let large_arc = rev_distance <= 3 || distance == 0;
    builder.line_to(in_point);
    builder.arc_to(
        radii,
        Angle::zero(),
        ArcFlags {
            large_arc,
            sweep: clockwise.reversed().into(),
        },
        out_point,
    );
    builder.line_to(to_location.get_point());
}

fn generate_qq_pp_curve(
    builder: &mut impl SvgPathBuilder,
    from_location: Location,
    to_location: Location,
    clockwise: Clockwise,
) {
    let start = from_location.get_point();
    let end = to_location.get_point();
    let sp = rotate_point(
        Point::origin(),
        start,
        Angle {
            radians: 67.5f32.to_radians() * if clockwise.into() { 1.0 } else { -1.0 },
        },
    );
    let sweep_table = [
        240.0f32, 268.0f32, 296.0f32, 326.0f32, 360.0f32, 406.0f32, 125.0f32, 209.0f32, 240.0f32,
    ]
    .map(|it| Angle {
        radians: it.to_radians() * if clockwise.into() { -1.0 } else { 1.0 },
    });
    let in_point = (start.to_vector() * 0.2).to_point();
    let arc_center = sp.lerp(Point::origin(), 0.5);
    let distance = from_location.tap_distance_to_with_clockwise(&to_location, clockwise);
    let sweep = sweep_table[usize::from(distance)];
    let p1 = rotate_point(arc_center, in_point, sweep / 2.0);
    let p2 = rotate_point(arc_center, p1, sweep / 2.0);
    let radii = in_point.distance_to(arc_center);
    let radii = vector(radii, radii);
    builder.line_to(in_point);
    builder.arc_to(
        radii,
        Angle::zero(),
        ArcFlags {
            large_arc: sweep.radians.abs() / 2.0 >= PI,
            sweep: clockwise.reversed().into(),
        },
        p1,
    );
    builder.arc_to(
        radii,
        Angle::zero(),
        ArcFlags {
            large_arc: sweep.radians.abs() / 2.0 >= PI,
            sweep: clockwise.reversed().into(),
        },
        p2,
    );
    builder.line_to(end);
}

fn generate_fan_line(builder: &mut impl SvgPathBuilder, to_location: Location) {
    generate_straight(builder, to_location);
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;
    use lyon_path::{
        ArcFlags, Path,
        geom::euclid::Transform2D,
        math::{Angle, point, vector},
        traits::SvgPathBuilder,
    };

    use crate::{
        deserialize::element::notes::{Location, LocationSector},
        deserialize::parser::path_generator::{
            generate_fan_line, generate_q_p_curve, generate_qq_pp_curve, generate_side_arc,
        },
        function_test_svg,
    };

    macro_rules! slide_circle_arc_test {
        ($name: ident, $location1: expr, $location2: expr, $cw: expr) => {
            function_test_svg!($name, test_circle_arc_slide, $location1, $location2, $cw);
        };
    }

    macro_rules! slide_side_arc_test {
        ($name: ident, $location1: expr, $location2: expr, $cw: expr) => {
            function_test_svg!($name, test_side_arc_slide, $location1, $location2, $cw);
        };
    }

    macro_rules! slide_qq_pp_arc_test {
        ($name: ident, $location1: expr, $location2: expr, $cw: expr) => {
            function_test_svg!(
                $name,
                test_outer_side_arc_slide,
                $location1,
                $location2,
                $cw
            );
        };
    }

    macro_rules! test_builder {
        () => {
            Path::builder()
                .transformed(Transform2D::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0))
                .with_svg()
        };
    }

    fn add_debug_circle(builder: &mut impl SvgPathBuilder) {
        builder.move_to(point(0.0, 0.5));
        builder.arc_to(
            vector(0.5, 0.5),
            Angle::zero(),
            ArcFlags {
                large_arc: true,
                sweep: false,
            },
            point(0.0, -0.5),
        );
        builder.arc_to(
            vector(0.5, 0.5),
            Angle::zero(),
            ArcFlags {
                large_arc: true,
                sweep: false,
            },
            point(0.0, 0.5),
        );
    }

    fn test_circle_arc_slide(index_1: u8, index_2: u8, clockwise: bool) -> String {
        let mut builder = test_builder!();
        let from_location = Location {
            sector: LocationSector::Tap,
            index: index_1,
        };
        let to_location = Location {
            sector: LocationSector::Tap,
            index: index_2,
        };
        builder.move_to(from_location.get_point());
        generate_q_p_curve(&mut builder, from_location, to_location, clockwise.into());
        add_debug_circle(&mut builder);
        output_builder(builder.build())
    }

    fn test_side_arc_slide(index_1: u8, index_2: u8, clockwise: bool) -> String {
        let mut builder = test_builder!();
        let sector = LocationSector::Tap;
        let from_location = Location {
            sector,
            index: index_1,
        };
        let to_location = Location {
            sector,
            index: index_2,
        };
        builder.move_to(from_location.get_point());
        generate_side_arc(&mut builder, from_location, to_location, clockwise.into());
        output_builder(builder.build())
    }

    fn test_outer_side_arc_slide(index_1: u8, index_2: u8, clockwise: bool) -> String {
        let mut builder = test_builder!();
        let sector = LocationSector::Tap;
        let from_location = Location {
            sector,
            index: index_1,
        };
        let to_location = Location {
            sector,
            index: index_2,
        };
        builder.move_to(from_location.get_point());
        generate_qq_pp_curve(&mut builder, from_location, to_location, clockwise.into());
        add_debug_circle(&mut builder);
        output_builder(builder.build())
    }

    fn test_outer_side_arc_slide_full(clockwise: bool) -> String {
        let mut builder = test_builder!();
        let sector = LocationSector::Tap;
        let from_location = Location { sector, index: 1 };
        for i in 1..=8 {
            builder.move_to(from_location.get_point());
            let to_location = Location { sector, index: i };
            generate_qq_pp_curve(&mut builder, from_location, to_location, clockwise.into());
        }
        add_debug_circle(&mut builder);
        output_builder(builder.build())
    }

    fn test_wifi_slide(index_1: u8, index_2: u8) -> String {
        let mut builder = test_builder!();
        let sector = LocationSector::Tap;
        let from_location = Location {
            sector,
            index: index_1,
        };
        let to_location = Location {
            sector,
            index: index_2,
        };
        builder.move_to(from_location.get_point());
        generate_fan_line(&mut builder, to_location);
        output_builder(builder.build())
    }

    fn output_builder(path: Path) -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"256.0\" height=\"256.0\" class=\"icon\" viewBox=\"-0.5 -0.5 1.0 1.0\"><path fill=\"none\" stroke=\"green\" stroke-width=\"0.01\" d={:?}/></svg>",
            path
        )
    }

    slide_circle_arc_test!(slide_center_circle_test_1, 1, 8, true);
    slide_circle_arc_test!(slide_center_circle_test_2, 1, 8, false);
    slide_circle_arc_test!(slide_center_circle_test_3, 8, 1, true);
    slide_circle_arc_test!(slide_center_circle_test_4, 8, 1, false);
    slide_circle_arc_test!(slide_center_circle_test_5, 1, 1, true);
    slide_circle_arc_test!(slide_center_circle_test_6, 1, 1, false);
    slide_circle_arc_test!(slide_center_circle_test_7, 1, 4, true);
    slide_circle_arc_test!(slide_center_circle_test_8, 4, 1, false);
    slide_side_arc_test!(slide_edge_test_1, 1, 2, true);
    slide_side_arc_test!(slide_edge_test_2, 1, 8, true);
    slide_side_arc_test!(slide_edge_test_3, 1, 8, false);
    slide_qq_pp_arc_test!(slide_qq_pp_test_1, 1, 8, true);
    slide_qq_pp_arc_test!(slide_qq_pp_test_2, 1, 8, false);
    slide_qq_pp_arc_test!(slide_qq_pp_test_3, 8, 1, true);
    slide_qq_pp_arc_test!(slide_qq_pp_test_4, 8, 1, false);
    slide_qq_pp_arc_test!(slide_qq_pp_test_5, 1, 1, true);
    slide_qq_pp_arc_test!(slide_qq_pp_test_6, 1, 1, false);
    slide_qq_pp_arc_test!(slide_qq_pp_test_7, 1, 4, true);
    slide_qq_pp_arc_test!(slide_qq_pp_test_8, 4, 1, false);
    function_test_svg!(
        slide_qq_pp_test_full_1,
        test_outer_side_arc_slide_full,
        false
    );
    function_test_svg!(
        slide_qq_pp_test_full_2,
        test_outer_side_arc_slide_full,
        true
    );
    function_test_svg!(slide_w_test_1, test_wifi_slide, 1, 5);
}
