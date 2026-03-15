use std::{
    f32::consts::PI,
    ops::{Deref, DerefMut},
};

use bitflags::bitflags;
use lyon_path::{
    Path,
    geom::euclid::point2,
    math::{Angle, Point},
};

use crate::deserialize::math::{
    Clockwise, get_circular_distance_with_direction, get_nearest_circular_distance,
    get_position_in_circle, move_circular,
};

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
pub enum LocationSector {
    A,
    B,
    C,
    D,
    E,
    #[default]
    Tap,
}

impl From<char> for LocationSector {
    fn from(value: char) -> Self {
        match value {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            'D' => Self::D,
            'E' => Self::E,
            _ => Self::Tap,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Location {
    pub sector: LocationSector,
    pub index: u8,
}

impl Location {
    pub fn is_sensor(&self) -> bool {
        self.sector != LocationSector::Tap
    }

    pub fn get_point(&self) -> Point {
        match &self.sector {
            LocationSector::C => point2(0.0, 0.0),
            LocationSector::B => get_position_in_circle(0.25, self.get_rad_angle()),
            LocationSector::D => get_position_in_circle(0.45, self.get_rad_angle()),
            LocationSector::E => get_position_in_circle(0.33, self.get_rad_angle()),
            LocationSector::A => get_position_in_circle(0.45, self.get_rad_angle()),
            LocationSector::Tap => get_position_in_circle(0.5, self.get_rad_angle()),
        }
    }

    pub fn get_rad_angle(&self) -> Angle {
        let radians = match &self.sector {
            LocationSector::C => 0.0,
            LocationSector::B => (PI * 5.0 / 8.0) + f32::from(self.index) * (-PI / 4.0),
            LocationSector::D => (PI / 2.0) + f32::from(self.index - 1) * (-PI / 4.0),
            LocationSector::E => (PI / 2.0) + f32::from(self.index - 1) * (-PI / 4.0),
            LocationSector::Tap | LocationSector::A => {
                (PI * 5.0 / 8.0) + f32::from(self.index) * (-PI / 4.0)
            }
        };
        Angle { radians }
    }

    pub fn tap_distance_to(&self, another: &Location) -> (u8, Clockwise) {
        if self.is_sensor() || another.is_sensor() {
            panic!("internal failure: not a tap location");
        }
        get_nearest_circular_distance(self.index, another.index, 8)
    }

    pub fn tap_distance_to_with_clockwise(&self, another: &Location, clockwise: Clockwise) -> u8 {
        if self.is_sensor() || another.is_sensor() {
            panic!("internal failure: not a tap location");
        }
        get_circular_distance_with_direction(self.index, another.index, 8, clockwise)
    }

    pub fn rotate(&self, offset: u8, clockwise: Clockwise) -> Location {
        Location {
            sector: self.sector,
            index: if self.sector == LocationSector::C {
                0
            } else {
                move_circular(self.index, offset, 8, clockwise)
            },
        }
    }
}

impl TryFrom<&str> for Location {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut chars = value.chars();
        let first_char = chars.next().ok_or("string is empty")?;
        let location_sector = LocationSector::from(first_char);
        if location_sector == LocationSector::Tap {
            let index = first_char.to_digit(10).ok_or(" not a valid number")?;
            if index == 0 || index > 8 {
                return Err("not a valid number");
            }
            Ok(Location {
                sector: location_sector,
                index: index as u8,
            })
        } else {
            let second_char = chars.next();
            match second_char {
                Some(second_char) => {
                    let index = second_char.to_digit(10).ok_or(" not a valid number")?;
                    Ok(Location {
                        sector: location_sector,
                        index: index as u8,
                    })
                }
                None => {
                    if location_sector == LocationSector::C {
                        Ok(Location {
                            sector: location_sector,
                            index: 0,
                        })
                    } else {
                        Err("no index specified")
                    }
                }
            }
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct NoteDecoration: u16 {
        const None=         0;
        const Break=        1 << 0;
        const Ex=           1 << 1;
        const Fireworks=    1 << 2;
        const Mine=         1 << 3;
        const SuddenIn=     1 << 4;
        const FadeOut=      1 << 5;
        const Each=         1 << 6;
        const ForceStar=    1 << 7;
        const Spinning=     1 << 8;
        const ForceNormal=  1 << 9;
    }
}

impl Default for NoteDecoration {
    fn default() -> Self {
        NoteDecoration::None
    }
}

#[derive(Debug)]
pub enum Note {
    Tap(NoteTap),
    Touch(NoteTouch),
    Hold(NoteHold),
    Slide(NoteSlide),
}

impl Deref for Note {
    type Target = NoteCommon;

    fn deref(&self) -> &Self::Target {
        match self {
            Note::Tap(note_tap) => &note_tap.common,
            Note::Touch(note_touch) => &note_touch.common,
            Note::Hold(note_hold) => &note_hold.common,
            Note::Slide(note_slide) => &note_slide.common,
        }
    }
}

impl DerefMut for Note {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Note::Tap(note_tap) => &mut note_tap.common,
            Note::Touch(note_touch) => &mut note_touch.common,
            Note::Hold(note_hold) => &mut note_hold.common,
            Note::Slide(note_slide) => &mut note_slide.common,
        }
    }
}

#[derive(Default)]
pub enum NoteType {
    #[default]
    Tap,
    Touch,
    Hold,
    Slide,
}

#[derive(Default, Debug)]
pub struct NoteCommon {
    pub location: Location,
    pub decoration: NoteDecoration,
}

#[derive(Debug)]
pub struct NoteTap {
    pub common: NoteCommon,
}

#[derive(Debug)]
pub struct NoteTouch {
    pub common: NoteCommon,
}

#[derive(Debug)]
pub struct NoteHold {
    pub common: NoteCommon,
    pub duration: f32,
}

#[derive(Debug)]
pub struct NoteSlide {
    pub common: NoteCommon,
    pub slide_branches: Vec<SlideBranch>,
}

#[derive(Default, Debug)]
pub struct SlideBranch {
    pub start_location: Location,
    pub slide_parts: Vec<SlidePart>,
}

#[derive(Debug)]
pub struct SlidePart {
    pub decoration: NoteDecoration,
    pub path: FractureRay,
    pub delay_duration: f32,
    pub duration: f32,
}

#[derive(Default)]
pub struct SlidePartRaw {
    pub location: SlideLocator,
    pub decoration: NoteDecoration,
}

#[derive(Debug)]
pub struct FractureRay {
    pub path: Path,
    pub length: f32,
}

#[derive(Default, Clone, Copy)]
pub enum SlideLocator {
    #[default]
    Invalid,
    Straight(Location),
    ClockwiseArc(Location),
    AntiClockwiseArc(Location),
    ShortArc(Location),
    VShape(Location),
    GrandVShape(Location, Location),
    ThunderboltS(Location),
    ThunderboltZ(Location),
    PShape(Location),
    QShape(Location),
    GrandPShape(Location),
    GrandQShape(Location),
    FanShape(Location),
}

impl SlideLocator {
    pub fn get_end_location(&self) -> Location {
        match &self {
            SlideLocator::Invalid => panic!("internal failure: invalid path"),
            SlideLocator::Straight(location)
            | SlideLocator::ClockwiseArc(location)
            | SlideLocator::AntiClockwiseArc(location)
            | SlideLocator::ShortArc(location)
            | SlideLocator::VShape(location)
            | SlideLocator::ThunderboltS(location)
            | SlideLocator::ThunderboltZ(location)
            | SlideLocator::PShape(location)
            | SlideLocator::QShape(location)
            | SlideLocator::GrandPShape(location)
            | SlideLocator::GrandQShape(location)
            | SlideLocator::FanShape(location)
            | SlideLocator::GrandVShape(_, location) => *location,
        }
    }
}

impl Deref for NoteTap {
    type Target = NoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for NoteTouch {
    type Target = NoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for NoteHold {
    type Target = NoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl Deref for NoteSlide {
    type Target = NoteCommon;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}
