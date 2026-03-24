use crate::{
    CHART_STATE, deserialize::chart::element::notes::NoteDecoration,
    render::state::element::NoteState,
};

#[derive(Default)]
pub struct NoteStatisticsPart {
    pub tap: u32,
    pub touch: u32,
    pub hold: u32,
    pub slide: u32,
    pub touch_hold: u32,
    pub each: u32,
    pub breaks: u32,
}

impl NoteStatisticsPart {
    fn append_count(&mut self, note_state: &NoteState) {
        let decoration = match note_state {
            NoteState::Tap(tap_note_state) => tap_note_state.note.decoration,
            NoteState::Touch(touch_note_state) => touch_note_state.note.decoration,
            NoteState::Hold(hold_note_state) => hold_note_state.note.decoration,
            NoteState::TouchHold(touch_hold_note_state) => touch_hold_note_state.note.decoration,
            NoteState::Slide(slide_note_state) => slide_note_state.note.decoration,
        };
        if decoration.contains(NoteDecoration::Each) {
            self.each += 1;
            return;
        }
        if decoration.contains(NoteDecoration::Break) {
            self.breaks += 1;
            return;
        }
        match note_state {
            NoteState::Tap(_) => {
                self.tap += 1;
            }
            NoteState::Touch(_) => {
                self.touch += 1;
            }
            NoteState::Hold(_) => {
                self.hold += 1;
            }
            NoteState::TouchHold(_) => {
                self.touch_hold += 1;
            }
            NoteState::Slide(_) => {
                self.slide += 1;
            }
        };
    }
}

pub fn get_estimated_total_time() -> f32 {
    CHART_STATE.with_borrow(|it| it.last().map(|it| it.time).unwrap_or(0.0))
}

pub fn get_note_counts(divisions: u16, total_duration: Option<f32>) -> Vec<NoteStatisticsPart> {
    let total_duration = total_duration.unwrap_or_else(get_estimated_total_time);
    CHART_STATE.with_borrow(|it| {
        let unit_duration = total_duration / f32::from(divisions);
        it.iter()
            .fold(
                (vec![NoteStatisticsPart::default()], 0.0f32, 0.0f32),
                |(mut vec, last_time, delta), it| {
                    let last = vec.last_mut().expect("internal failure");
                    last.append_count(it);
                    let delta = delta + it.time - last_time;
                    if delta >= unit_duration {
                        vec.push(NoteStatisticsPart::default());
                        (vec, it.time, delta - unit_duration)
                    } else {
                        (vec, it.time, delta)
                    }
                },
            )
            .0
    })
}
