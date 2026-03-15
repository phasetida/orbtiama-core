use crate::deserialize::element::notes::Note;

#[derive(Default, Clone, Debug)]
pub struct Timing {
    pub time: f32,
    pub tempo: f32,
    pub subdivisions: f32,
}

impl Timing {
    pub fn get_seconds_per_bar(&self) -> f32 {
        if self.tempo == 0.0 {
            0.0
        } else {
            60.0 / self.tempo
        }
    }

    pub fn get_seconds_per_beat(&self) -> f32 {
        self.get_seconds_per_bar()
            / ((if self.subdivisions == 0.0 {
                4.0
            } else {
                self.subdivisions
            }) / 4.0)
    }

    pub fn set_second(&mut self, second: f32) {
        self.tempo = 60.0 / second;
        self.subdivisions = 4.0;
    }
}

#[derive(Default, Debug)]
pub enum EachStyle {
    #[default]
    Default,
    Broken,
}

#[derive(Default, Debug)]
pub struct NoteCollection {
    pub time: f32,
    pub notes: Vec<Note>,
}

#[derive(Default, Debug)]
pub struct Chart {
    pub note_collections: Vec<NoteCollection>,
    pub timings: Vec<Timing>,
}
