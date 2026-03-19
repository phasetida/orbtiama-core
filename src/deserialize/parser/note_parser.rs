use std::iter;

use pest::iterators::Pair;

use crate::deserialize::{
    element::{
        chart::{NoteCollection, Timing},
        notes::{
            Location, Note, NoteCommon, NoteDecoration, NoteHold, NoteSlide, NoteTap, NoteTouch,
            SlideBranch, SlideBranchRaw, SlideLocator, SlidePart, SlidePartRaw,
        },
    },
    lexer::Rule,
    math::{Clockwise, linearize_path},
    parser::{
        path_generator::generate_path,
        time_parser::{read_duration_hold, read_duration_slide, read_duration_slide_with_delay},
    },
};

pub fn parse_note_collection(pair: Pair<Rule>, timings: &[Timing], time: f32) -> NoteCollection {
    let mut note_collection = NoteCollection {
        time,
        ..NoteCollection::default()
    };
    let mut each_types = Vec::<bool>::new();
    for it in pair.into_inner() {
        match it.as_rule() {
            Rule::tap_note => note_collection.notes.push(read_tap_note(it)),
            Rule::touch_note => note_collection.notes.push(read_touch_note(it)),
            Rule::hold_note | Rule::touch_hold_note => {
                let latest_timing = timings
                    .last()
                    .expect("parser_note_collection: internal failure");
                note_collection
                    .notes
                    .push(read_hold_note(it, latest_timing));
            }
            Rule::slide_note => {
                let latest_timing = timings
                    .last()
                    .expect("parser_note_collection: internal failure");
                note_collection
                    .notes
                    .push(read_slide_note(it, latest_timing));
            }
            Rule::each_divider_fake => {
                each_types.push(false);
            }
            Rule::each_divider_normal => {
                each_types.push(true);
            }
            _ => panic!(
                "parser_note_collection: expect note or division, found {:?}",
                it.as_str()
            ),
        };
    }
    process_each(&mut note_collection.notes, &each_types);
    note_collection
}

fn process_each(notes: &mut [Note], each: &[bool]) {
    for (i, it) in each.iter().enumerate() {
        if !*it {
            continue;
        }
        notes[i].decoration |= NoteDecoration::Each;
        notes[i + 1].decoration |= NoteDecoration::Each;
    }
    process_slide_each(notes);
}

fn process_slide_each(notes: &mut [Note]) {
    let slides: Vec<_> = notes
        .iter_mut()
        .filter_map(|it| match it {
            Note::Slide(note_slide) => Some(note_slide),
            _ => None,
        })
        .collect();
    let each = slides.len() > 1;
    if !each {
        return;
    }
    slides.into_iter().for_each(|it| {
        it.common.decoration |= NoteDecoration::Each;
        it.slide_branches.iter_mut().for_each(|it| {
            it.slide_parts.iter_mut().for_each(|it| {
                it.decoration |= NoteDecoration::Each;
                it.decoration |= NoteDecoration::TailEach;
            });
        });
    });
}

fn read_tap_note(pair: Pair<Rule>) -> Note {
    Note::Tap(NoteTap {
        common: read_common(pair),
    })
}

fn read_touch_note(pair: Pair<Rule>) -> Note {
    Note::Touch(NoteTouch {
        common: read_common(pair),
    })
}

fn read_hold_note(pair: Pair<Rule>, latest_timing: &Timing) -> Note {
    let mut iter = pair.into_inner();
    let mut common = NoteCommon {
        location: iter
            .next()
            .as_ref()
            .map(read_location)
            .expect("parser_note_hold: expect location, found nothing"),
        ..Default::default()
    };
    let mut length = 0.0;
    for it in iter {
        if read_common_single(&it, &mut common) {
            continue;
        }
        match it.as_rule() {
            Rule::decoration_hold => {}
            Rule::duration_hold => {
                length = read_duration_hold(it, latest_timing);
            }
            _ => panic!(
                "parse_note_hold: unexpected token: {:?}: {:?}",
                it.as_str(),
                it.as_rule()
            ),
        }
    }
    Note::Hold(NoteHold {
        common,
        duration: length,
    })
}

fn read_slide_note(pair: Pair<Rule>, latest_timing: &Timing) -> Note {
    let mut iter = pair.into_inner();
    let mut common = NoteCommon {
        location: iter
            .next()
            .as_ref()
            .map(read_location)
            .expect("parser_slide: expect location, found nothing"),
        ..Default::default()
    };
    let mut slide_branches_raw = Vec::<SlideBranchRaw>::new();
    for it in iter {
        if read_common_single(&it, &mut common) {
            continue;
        }
        match it.as_rule() {
            Rule::slide_branch => {
                slide_branches_raw.push(read_slide_branch_raw(it, common.location, latest_timing));
            }
            Rule::slide_joiner => {}
            _ => panic!("parser_slide: unexpected token: {:?}", it.as_str()),
        }
    }
    process_slide_branch_each(&mut slide_branches_raw);
    let slide_branches_raw = process_slide_branch_wifi(slide_branches_raw);
    let slide_branches: Vec<_> = slide_branches_raw
        .into_iter()
        .map(|it| {
            let mut parts = cook_slide_parts_raw(it.start_location, &it.slide_parts);
            process_slide_parts_break(&mut parts);
            SlideBranch {
                start_location: it.start_location,
                slide_parts: parts,
            }
        })
        .collect();
    if slide_branches.len() > 1 {
        common.decoration |= NoteDecoration::DoubleStar;
    }
    Note::Slide(NoteSlide {
        common,
        slide_branches,
    })
}

fn process_slide_branch_wifi(raw: Vec<SlideBranchRaw>) -> Vec<SlideBranchRaw> {
    raw.into_iter()
        .flat_map(|it| {
            if let Some(part) = it.slide_parts.first().cloned()
                && let SlideLocator::FanShape(location) = part.0.location
            {
                let start_location = it.start_location;
                let mut decoration = part.0.decoration;
                decoration.remove(NoteDecoration::WifiSlide);
                vec![
                    it,
                    SlideBranchRaw {
                        start_location,
                        slide_parts: vec![(
                            SlidePartRaw {
                                location: SlideLocator::Straight(location.rotate(1, Clockwise::Cw)),
                                decoration,
                            },
                            part.1,
                        )],
                    },
                    SlideBranchRaw {
                        start_location,
                        slide_parts: vec![(
                            SlidePartRaw {
                                location: SlideLocator::Straight(
                                    location.rotate(1, Clockwise::Ccw),
                                ),
                                decoration,
                            },
                            part.1,
                        )],
                    },
                ]
            } else {
                vec![it]
            }
        })
        .collect()
}

fn read_slide_branch_raw(
    pair: Pair<Rule>,
    start_location: Location,
    latest_timing: &Timing,
) -> SlideBranchRaw {
    let mut parts = Vec::<(SlidePartRaw, Option<(f32, f32)>)>::new();
    for it in pair.into_inner() {
        match it.as_rule() {
            Rule::slide_part | Rule::slide_part_with_time => {
                parts.push(read_slide_part(it, latest_timing));
            }
            Rule::duration_slide_with_delay => {
                let duration = read_duration_slide_with_delay(it, latest_timing);
                let slide_part = parts.pop().expect("parser_slide_branch: internal failure");
                parts.push((slide_part.0, Some(duration)));
            }
            _ => panic!(
                "parser_slide_branch: unexpected token: {:?}: {:?}",
                it.as_str(),
                it.as_rule()
            ),
        }
    }
    SlideBranchRaw {
        start_location,
        slide_parts: parts,
    }
}

fn process_slide_parts_break(parts: &mut [SlidePart]) {
    let is_tail_break = parts
        .iter()
        .any(|it| it.decoration.contains(NoteDecoration::Break));
    if !is_tail_break {
        return;
    }
    for it in parts {
        it.decoration |= NoteDecoration::Break;
        it.decoration |= NoteDecoration::TailBreak;
    }
}

fn process_slide_branch_each(branches: &mut [SlideBranchRaw]) {
    if branches.len() <= 1 {
        return;
    }
    branches.iter_mut().for_each(|it| {
        it.slide_parts.iter_mut().for_each(|it| {
            it.0.decoration |= NoteDecoration::Each;
            it.0.decoration |= NoteDecoration::TailEach;
        });
    });
}

#[deny(clippy::pedantic)]
fn cook_slide_parts_raw(
    start_location: Location,
    raw: &[(SlidePartRaw, Option<(f32, f32)>)],
) -> Vec<SlidePart> {
    if raw.iter().all(|(_, time)| time.is_some()) {
        return (iter::once(None).chain(raw.iter().map(Option::from)))
            .zip(raw.iter().map(Option::from).chain(iter::once(Option::None)))
            .filter_map(|it| {
                let (last, current) = it;
                if let Some(current) = current {
                    let (raw, time) = current;
                    let (duration, delay) =
                        time.expect("parser_slide_part_raw: internal failure #2");
                    let location = raw.location;
                    if let Some(last) = last {
                        Some(SlidePart {
                            decoration: raw.decoration,
                            path: linearize_path(generate_path(
                                last.0.location.get_end_location(),
                                location,
                            )),
                            delay_duration: 0.0,
                            duration,
                        })
                    } else {
                        Some(SlidePart {
                            decoration: raw.decoration,
                            path: linearize_path(generate_path(start_location, location)),
                            delay_duration: delay,
                            duration,
                        })
                    }
                } else {
                    None
                }
            })
            .collect();
    }
    if let Some((_, Some((duration, delay)))) = raw.iter().last() {
        let tmp: Vec<_> = (iter::once(None).chain(raw.iter().map(Option::from)))
            .zip(raw.iter().map(Option::from).chain(iter::once(Option::None)))
            .filter_map(|it| {
                let (last, current) = it;
                if let Some(current) = current {
                    let (raw, _) = current;
                    let location = raw.location;
                    if let Some(last) = last {
                        Some(SlidePart {
                            decoration: raw.decoration,
                            path: linearize_path(generate_path(
                                last.0.location.get_end_location(),
                                location,
                            )),
                            delay_duration: 0.0,
                            duration: 0.0,
                        })
                    } else {
                        Some(SlidePart {
                            decoration: raw.decoration,
                            path: linearize_path(generate_path(start_location, location)),
                            delay_duration: *delay,
                            duration: 0.0,
                        })
                    }
                } else {
                    None
                }
            })
            .collect();
        let total_length = tmp.iter().fold(0.0, |i, it| i + it.path.length);
        return tmp
            .into_iter()
            .map(|it| SlidePart {
                duration: duration * it.path.length / total_length,
                ..it
            })
            .collect();
    }
    panic!("parser_slide_part_raw: invalid slide branch");
}

fn read_slide_part(pair: Pair<Rule>, latest_timing: &Timing) -> (SlidePartRaw, Option<(f32, f32)>) {
    let mut decoration = NoteDecoration::default();
    let mut iter = pair.into_inner();
    let location = iter
        .next()
        .map(read_slide_location)
        .expect("parser_slide_part: expect slide location, found nothing");
    let mut time = Option::<(f32, f32)>::None;
    for it in iter {
        if read_common_decoration(&it, &mut decoration) {
            continue;
        }
        match it.as_rule() {
            Rule::duration_slide => {
                time = Some(read_duration_slide(it, latest_timing));
            }
            Rule::duration_slide_with_delay => {
                time = Some(read_duration_slide_with_delay(it, latest_timing));
            }
            _ => panic!("parser_slide_part: unexpected token: {:?}", it.as_str()),
        };
    }
    if matches!(location, SlideLocator::FanShape(_)) {
        decoration |= NoteDecoration::WifiSlide;
    }
    (
        SlidePartRaw {
            location,
            decoration,
        },
        time,
    )
}

fn read_slide_location(pair: Pair<Rule>) -> SlideLocator {
    let mut iter = pair.into_inner();
    let pair = iter
        .next()
        .expect("parser_slide_location: expect location shell, found nothing");
    match pair.as_rule() {
        Rule::slide_normal_locator => {
            let mut inner_iter = pair.into_inner();
            let locator_pair = inner_iter
                .next()
                .expect("parser_slide_location: expect locator found nothing");
            let location_pair = iter
                .next()
                .expect("parser_slide_location: expect location, found nothing");
            read_slide_locator(&locator_pair, read_location(&location_pair))
        }
        Rule::slide_grand_v_locator => {
            let mut inner_iter = pair.into_inner();
            let locator_pair = inner_iter
                .next()
                .expect("parser_slide_location: expect grand V locator, found nothing");
            if locator_pair.as_rule() != Rule::slide_locator_grand_v {
                panic!(
                    "parser_slide_location: expect grand V locator, found {:?}",
                    locator_pair.as_str()
                )
            }
            let location1 = inner_iter
                .next()
                .as_ref()
                .map(read_location)
                .expect("parser_slide_location: expect location 1, found nothing");
            let location2 = inner_iter
                .next()
                .as_ref()
                .map(read_location)
                .expect("parser_slide_location: expect location 2, found nothing");
            SlideLocator::GrandVShape(location1, location2)
        }
        _ => panic!(
            "parser_slide_location: unexpected token: {:?}",
            pair.as_str()
        ),
    }
}

fn read_common(pair: Pair<Rule>) -> NoteCommon {
    let mut iter = pair.into_inner();
    let mut common = NoteCommon {
        location: iter
            .next()
            .as_ref()
            .map(read_location)
            .expect("parser_common: expect location, found nothing"),
        ..NoteCommon::default()
    };
    for it in iter {
        if !read_common_single(&it, &mut common) {
            panic!("parser_common: unexpected token: {:?}", it.as_str());
        }
    }
    common
}

fn read_common_single(pair: &Pair<Rule>, common: &mut NoteCommon) -> bool {
    match pair.as_rule() {
        Rule::location => common.location = read_location(pair),
        r if read_common_decoration(pair, &mut common.decoration) => {}
        _ => return false,
    };
    true
}

fn read_common_decoration(pair: &Pair<Rule>, decoration: &mut NoteDecoration) -> bool {
    match pair.as_rule() {
        Rule::decoration_break => *decoration |= NoteDecoration::Break,
        Rule::decoration_ex => *decoration |= NoteDecoration::Ex,
        Rule::decoration_firework => *decoration |= NoteDecoration::Fireworks,
        Rule::decoration_fade_out => *decoration |= NoteDecoration::FadeOut,
        Rule::decoration_sudden_in => *decoration |= NoteDecoration::SuddenIn,
        Rule::decoration_spinning_star => {
            *decoration |= NoteDecoration::ForceStar;
            *decoration |= NoteDecoration::Spinning;
        }
        Rule::decoration_star => *decoration |= NoteDecoration::ForceStar,
        Rule::decoration_force_normal => *decoration |= NoteDecoration::ForceNormal,
        _ => return false,
    };
    true
}

fn read_slide_locator(pair: &Pair<Rule>, location: Location) -> SlideLocator {
    match pair.as_rule() {
        Rule::slide_locator_straight => SlideLocator::Straight(location),
        Rule::slide_locator_clockwise_arc => SlideLocator::ClockwiseArc(location),
        Rule::slide_locator_anti_clockwise_arc => SlideLocator::AntiClockwiseArc(location),
        Rule::slide_locator_short_arc => SlideLocator::ShortArc(location),
        Rule::slide_locator_v => SlideLocator::VShape(location),
        Rule::slide_locator_thunderbolt_s => SlideLocator::ThunderboltS(location),
        Rule::slide_locator_thunderbolt_z => SlideLocator::ThunderboltZ(location),
        Rule::slide_locator_q => SlideLocator::QShape(location),
        Rule::slide_locator_p => SlideLocator::PShape(location),
        Rule::slide_locator_grand_p => SlideLocator::GrandPShape(location),
        Rule::slide_locator_grand_q => SlideLocator::GrandQShape(location),
        Rule::slide_locator_fan => SlideLocator::FanShape(location),
        _ => panic!(
            "parser_slide_locator: unexpected token: {:?}",
            pair.as_str()
        ),
    }
}

fn read_location(pair: &Pair<Rule>) -> Location {
    Location::try_from(pair.as_str())
        .unwrap_or_else(|e| panic!("parser_location: parse failure: {:?}", e))
}
