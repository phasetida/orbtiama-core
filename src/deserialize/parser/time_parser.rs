use pest::iterators::Pair;

use crate::{deserialize::element::chart::Timing, deserialize::lexer::Rule};

pub fn parse_tempo_change(pair: Pair<Rule>, timings: &mut Vec<Timing>, time: f32) {
    let str = pair
        .into_inner()
        .find(|it| it.as_rule() == Rule::tempo)
        .expect("parser_tempo: expect tempo, found nothing")
        .as_str();
    let tempo = str
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("parser_tempo: expect tempo, found {:?}", str));
    update_last_timing(timings, |it| Timing { time, tempo, ..it });
}

pub fn parse_subdivision_change(pair: Pair<Rule>, timings: &mut Vec<Timing>, time: f32) {
    let pair = pair
        .into_inner()
        .next()
        .expect("parser_subdivision: expect subdivision, found nothing");
    let inner_pair = pair
        .into_inner()
        .next()
        .expect("parser_subdivision: expect subdivision, found inner nothing");
    let subdivision_str = inner_pair.as_str();
    let subdivision = subdivision_str.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "parser_subdivision: expect number, found {:?}",
            subdivision_str
        )
    });
    match inner_pair.as_rule() {
        Rule::subdivision_absolute => {
            update_last_timing(timings, |_| Timing {
                time,
                tempo: 60.0 / subdivision,
                subdivisions: 4.0,
            });
        }
        Rule::subdivision_beat => {
            update_last_timing(timings, |it| Timing {
                time,
                subdivisions: subdivision,
                ..it
            });
        }
        _ => panic!(
            "parser_division: expect subdivision, found {:?}",
            inner_pair.as_str()
        ),
    };
}

pub fn parse_time_step(timings: &[Timing]) -> f32 {
    timings
        .last()
        .expect("parser_time_step: internal failure")
        .get_seconds_per_beat()
}

fn update_last_timing(timings: &mut Vec<Timing>, timing_modifier: impl Fn(Timing) -> Timing) {
    let last_timing = timings.last().expect("timing_update: internal failure");
    let modified_timing = timing_modifier(last_timing.clone());
    if f32_peq(last_timing.time, modified_timing.time) {
        timings.pop();
    }
    timings.push(modified_timing);
}

fn f32_peq(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

fn read_duration_bpm(pair: Pair<Rule>, latest_timing: &Timing) -> (f32, Timing) {
    let iter = pair.into_inner();
    let bpm = iter
        .clone()
        .find(|it| it.as_rule() == Rule::bpm)
        .and_then(|it| it.as_str().parse::<f32>().ok())
        .unwrap_or(latest_timing.tempo);
    let denominator_str = iter
        .clone()
        .find(|it| it.as_rule() == Rule::denominator)
        .expect("parser_duration_hold: expect denominator, found nothing")
        .as_str();
    let numerator_str = iter
        .clone()
        .find(|it| it.as_rule() == Rule::numerator)
        .expect("parser_duration_hold: expect numerator, found nothing")
        .as_str();
    let numerator = numerator_str.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "parser_duration_hold: expect numerator, found {:?}",
            numerator_str
        )
    });
    let denominator = denominator_str.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "parser_duration_hold: expect denominator, found {:?}",
            denominator_str
        )
    });
    let timing = Timing {
        tempo: bpm,
        ..latest_timing.clone()
    };
    (
        timing.get_seconds_per_bar() / (denominator / 4.0) * numerator,
        timing,
    )
}

pub fn read_duration_hold(pair: Pair<Rule>, latest_timing: &Timing) -> f32 {
    let inner_pair = pair
        .into_inner()
        .next()
        .expect("parser_duration_hold: expect duration_hold, found nothing");
    match inner_pair.as_rule() {
        Rule::duration_with_bpm => read_duration_bpm(inner_pair, latest_timing).0,
        Rule::duration_absolute => {
            let absolute_str = inner_pair.as_str();
            absolute_str.parse::<f32>().unwrap_or_else(|_| {
                panic!(
                    "parser_duration_hold: expect absolute duration, found {:?}",
                    absolute_str
                )
            })
        }
        _ => panic!(
            "parser_duration_hold: expect duration(abs or rel), found {:?}",
            inner_pair.as_str()
        ),
    }
}

pub fn read_duration_slide_with_delay(pair: Pair<Rule>, latest_timing: &Timing) -> (f32, f32) {
    let mut delay = latest_timing.get_seconds_per_bar();
    let mut duration = 0.0;
    let iter = pair.into_inner();
    if iter.is_empty() {
        panic!("parser_duration_slide: expect duration, found nothing");
    }
    for it in iter {
        match it.as_rule() {
            Rule::delay_duration_absolute => {
                delay = it.as_str().parse::<f32>().unwrap_or_else(|_| {
                    panic!(
                        "parser_duration_slide: expect number, found: {:?}",
                        it.as_str()
                    )
                });
            }
            Rule::duration_with_bpm => {
                let (new_duration, timing) = read_duration_bpm(it, latest_timing);
                duration = new_duration;
                delay = timing.get_seconds_per_bar();
            }
            Rule::duration_absolute => {
                duration = it.as_str().parse::<f32>().unwrap_or_else(|_| {
                    panic!(
                        "parser_duration_slide: expect number, found: {:?}",
                        it.as_str()
                    )
                });
            }
            _ => panic!("parser_duration_slide: unexpected token: {:?}", it.as_str()),
        };
    }
    (duration, delay)
}

pub fn read_duration_slide(pair: Pair<Rule>, latest_timing: &Timing) -> (f32, f32) {
    pair.into_inner()
        .next()
        .map(|it| read_duration_bpm(it, latest_timing))
        .map(|(duration, timing)| (duration, timing.get_seconds_per_bar()))
        .expect("parser_slide_duration: expect duration, found nothing")
}
