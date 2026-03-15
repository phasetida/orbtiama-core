use pest::iterators::Pairs;

use crate::{
    deserialize::lexer::Rule,
    deserialize::element::chart::{Chart, NoteCollection, Timing},
};

pub mod note_parser;
pub mod time_parser;
pub mod path_generator;

pub fn parse_chart(pairs: Pairs<Rule>) -> Chart {
    let root_pair = pairs
        .into_iter()
        .next()
        .expect("parser: expect chart, found nothing");
    if root_pair.as_rule() != Rule::chart {
        panic!("parser: expect chart, found {:?}", root_pair);
    }
    let mut note_collections = Vec::<NoteCollection>::new();
    let mut timings = Vec::<Timing>::new();
    let mut time = 0.0;
    timings.push(Timing::default());
    for it in root_pair.into_inner() {
        let rule = it.as_rule();
        match rule {
            Rule::tempo_change => time_parser::parse_tempo_change(it, &mut timings, time),
            Rule::subdivision_change => {
                time_parser::parse_subdivision_change(it, &mut timings, time)
            }
            Rule::note_collection => {
                note_collections.push(note_parser::parse_note_collection(it, &timings, time))
            }
            Rule::time_step => time += time_parser::parse_time_step(&timings),
            Rule::COMMENT | Rule::WHITESPACE => {}
            _ => panic!("parser: unexpected rule {:?}", rule),
        };
    }
    Chart {
        note_collections,
        timings,
    }
}
