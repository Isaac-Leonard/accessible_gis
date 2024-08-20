use itertools::Itertools;
use strum::{EnumIter, IntoEnumIterator};

pub struct AudioSettings {
    pub min_freq: f64,
    pub max_freq: f64,
    pub volume: f64,
    no_data_value_sound: AudioIndicator,
    border_sound: AudioIndicator,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            min_freq: 220.0,
            max_freq: 880.0,
            volume: 1.0,
            no_data_value_sound: AudioIndicator::Different,
            border_sound: AudioIndicator::MinFreq,
        }
    }
}

#[derive(EnumIter)]
pub enum AudioIndicator {
    Silence,
    MinFreq,
    MaxFreq,
    Verbal,
    Different,
}

impl AudioIndicator {
    fn get_all_options() -> Vec<Self> {
        Self::iter().collect_vec()
    }
}
