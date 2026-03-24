use crate::{
    CHART_STATE,
    deserialize::chart::{
        element::{chart::Chart, notes::Note},
        lexer::ChartLexer,
        parser::ChartParser,
    },
    render::state::element::{
        HoldNoteState, NoteState, NoteStateCommon, SlideBranchState, SlideNoteState, TapNoteState,
        TouchHoldNoteState, TouchNoteState,
    },
};

impl From<Chart> for Vec<NoteState> {
    fn from(value: Chart) -> Self {
        convert_state(value)
    }
}

pub fn initialize_state_direct(str: &str) {
    let result = ChartLexer::raw_parse_str(str).expect("failed to load chart");
    let result = ChartParser::parse_chart(result);
    initialize_state(result);
}

pub fn initialize_state(chart: Chart) {
    CHART_STATE.with_borrow_mut(|it| {
        *it = convert_state(chart);
    });
}

fn convert_state(chart: Chart) -> Vec<NoteState> {
    chart
        .note_collections
        .into_iter()
        .flat_map(|collection| {
            let time = collection.time;
            collection
                .notes
                .into_iter()
                .map(move |it| initialize_note_state(it, time))
        })
        .collect()
}

fn initialize_note_state(note: Note, time: f32) -> NoteState {
    let common = NoteStateCommon {
        time,
        ..NoteStateCommon::default()
    };
    let hint_rotate = -note
        .location
        .get_point()
        .to_vector()
        .angle_from_x_axis()
        .radians;
    match note {
        Note::Tap(note_tap) => NoteState::Tap(TapNoteState {
            common,
            note: note_tap,
            hint_scale: 0.0,
            hint_rotate,
            hint_alpha: 0.0,
        }),
        Note::Touch(note_touch) => NoteState::Touch(TouchNoteState {
            common,
            note: note_touch,
            progress: 0.0,
        }),
        Note::Hold(note_hold) => {
            if note_hold.location.is_sensor() {
                NoteState::TouchHold(TouchHoldNoteState {
                    common,
                    show_progress: 0.0,
                    hold_progress: 0.0,
                    note: note_hold,
                })
            } else {
                NoteState::Hold(HoldNoteState {
                    common,
                    note: note_hold,
                    holding: false,
                    tail_x: 0.0,
                    tail_y: 0.0,
                    hint_rotate,
                    hint_scale: 0.0,
                    hint_alpha: 0.0,
                })
            }
        }
        Note::Slide(note_slide) => {
            let len = note_slide.slide_branches.len();
            NoteState::Slide(SlideNoteState {
                common,
                head_progress: 0.0,
                body_alpha: 0.0,
                note: note_slide,
                branch_states: (0..len).map(|_| SlideBranchState::default()).collect(),
                hint_rotate,
                hint_scale: 0.0,
                hint_alpha: 0.0,
            })
        }
    }
}
