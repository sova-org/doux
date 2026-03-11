//! 3-band DJ-style EQ using shelving and peaking filters.

use crate::dsp::Biquad;
use crate::types::FilterType;

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
    pub fn process(
        &mut self,
        input: f32,
        lo_db: f32,
        mid_db: f32,
        hi_db: f32,
        lo_freq: f32,
        mid_freq: f32,
        hi_freq: f32,
        sr: f32,
    ) -> f32 {
        let mut out = input;
        if lo_db != 0.0 {
            out = self
                .lo
                .process_with_gain(out, FilterType::Lowshelf, lo_freq, SHELF_Q, lo_db, sr);
        }
        if mid_db != 0.0 {
            out = self
                .mid
                .process_with_gain(out, FilterType::Peaking, mid_freq, MID_Q, mid_db, sr);
        }
        if hi_db != 0.0 {
            out = self
                .hi
                .process_with_gain(out, FilterType::Highshelf, hi_freq, SHELF_Q, hi_db, sr);
        }
        out
    }
}
