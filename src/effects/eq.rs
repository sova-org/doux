//! 3-band DJ-style EQ using shelving and peaking filters.

use crate::dsp::Biquad;
use crate::types::FilterType;

const LO_FREQ: f32 = 200.0;
const MID_FREQ: f32 = 1000.0;
const HI_FREQ: f32 = 5000.0;
const MID_Q: f32 = 0.7;
const SHELF_Q: f32 = 0.707;

/// 3-band EQ: low shelf, mid peak, high shelf.
#[derive(Clone, Copy, Default)]
pub struct Eq {
    lo: Biquad,
    mid: Biquad,
    hi: Biquad,
}

impl Eq {
    /// Process one sample. Gains are in dB (0.0 = bypass).
    #[inline]
    pub fn process(&mut self, input: f32, lo_db: f32, mid_db: f32, hi_db: f32, sr: f32) -> f32 {
        let mut out = input;
        if lo_db != 0.0 {
            out = self
                .lo
                .process_with_gain(out, FilterType::Lowshelf, LO_FREQ, SHELF_Q, lo_db, sr);
        }
        if mid_db != 0.0 {
            out = self
                .mid
                .process_with_gain(out, FilterType::Peaking, MID_FREQ, MID_Q, mid_db, sr);
        }
        if hi_db != 0.0 {
            out =
                self.hi
                    .process_with_gain(out, FilterType::Highshelf, HI_FREQ, SHELF_Q, hi_db, sr);
        }
        out
    }
}
