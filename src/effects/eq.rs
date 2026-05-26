//! 3-band DJ-style EQ using shelving and peaking filters.

use crate::dsp::Biquad;
use crate::types::{FilterType, ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "eq",
    description: "3-band parametric EQ (low shelf, mid peak, high shelf)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "eqlo",
            aliases: &[],
            description: "low shelf gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqmid",
            aliases: &[],
            description: "mid peak gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqhi",
            aliases: &[],
            description: "high shelf gain in dB",
            default: "0.0",
            min: -24.0,
            max: 24.0,
        },
        ParamInfo {
            name: "eqlofreq",
            aliases: &[],
            description: "low shelf frequency in Hz",
            default: "200.0",
            min: 20.0,
            max: 2000.0,
        },
        ParamInfo {
            name: "eqmidfreq",
            aliases: &[],
            description: "mid peak frequency in Hz",
            default: "1000.0",
            min: 100.0,
            max: 10000.0,
        },
        ParamInfo {
            name: "eqhifreq",
            aliases: &[],
            description: "high shelf frequency in Hz",
            default: "5000.0",
            min: 1000.0,
            max: 20000.0,
        },
    ],
};

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
    #[inline]
    #[allow(clippy::too_many_arguments)]
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
        let mut x = input;
        if lo_db != 0.0 {
            x = self
                .lo
                .process_with_gain(x, FilterType::Lowshelf, lo_freq, SHELF_Q, lo_db, sr);
        }
        if mid_db != 0.0 {
            x = self
                .mid
                .process_with_gain(x, FilterType::Peaking, mid_freq, MID_Q, mid_db, sr);
        }
        if hi_db != 0.0 {
            x = self
                .hi
                .process_with_gain(x, FilterType::Highshelf, hi_freq, SHELF_Q, hi_db, sr);
        }
        x
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        lo_db: f32,
        mid_db: f32,
        hi_db: f32,
        lo_freq: f32,
        mid_freq: f32,
        hi_freq: f32,
        sr: f32,
    ) {
        if lo_db != 0.0 {
            self.lo.process_block_with_gain(
                buf,
                n,
                ch,
                FilterType::Lowshelf,
                lo_freq,
                SHELF_Q,
                lo_db,
                sr,
            );
        }
        if mid_db != 0.0 {
            self.mid.process_block_with_gain(
                buf,
                n,
                ch,
                FilterType::Peaking,
                mid_freq,
                MID_Q,
                mid_db,
                sr,
            );
        }
        if hi_db != 0.0 {
            self.hi.process_block_with_gain(
                buf,
                n,
                ch,
                FilterType::Highshelf,
                hi_freq,
                SHELF_Q,
                hi_db,
                sr,
            );
        }
    }
}
