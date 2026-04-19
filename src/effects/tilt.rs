//! Single-knob tilt EQ using a high shelf filter.

use crate::dsp::Biquad;
use crate::types::{FilterType, ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "tilt",
    description: "Single-knob spectral balance tilt EQ",
    group: ModuleGroup::Effect,
    params: &[ParamInfo {
        name: "tilt",
        aliases: &[],
        description: "spectral balance (-1 dark, 0 flat, 1 bright)",
        default: "0.0",
        min: -1.0,
        max: 1.0,
    }],
};

const TILT_FREQ: f32 = 800.0;
const TILT_Q: f32 = 0.707;
const MAX_DB: f32 = 6.0;

/// Tilt EQ: one knob shifts spectral balance.
#[derive(Clone, Copy, Default)]
pub struct Tilt {
    shelf: Biquad,
}

impl Tilt {
    /// Process one sample. tilt ranges from -1.0 (dark) to 1.0 (bright).
    #[inline]
    pub fn process(&mut self, input: f32, tilt: f32, sr: f32) -> f32 {
        let db = tilt.clamp(-1.0, 1.0) * MAX_DB;
        self.shelf
            .process_with_gain(input, FilterType::Highshelf, TILT_FREQ, TILT_Q, db, sr)
    }
}
