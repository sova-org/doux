//! Sample rate reduction (bitcrusher-style decimation).
//!
//! Reduces the effective sample rate by holding each sample value for multiple
//! output samples, creating the characteristic "crunchy" lo-fi sound of early
//! samplers and video game consoles.
//!
//! # Example
//!
//! With `factor = 4` at 48kHz, the effective sample rate becomes 12kHz:
//!
//! ```text
//! Input:  [a, b, c, d, e, f, g, h, ...]
//! Output: [a, a, a, a, e, e, e, e, ...]
//! ```

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "coarse",
    description: "Sample rate reduction (decimation)",
    group: ModuleGroup::Effect,
    params: &[ParamInfo {
        name: "coarse",
        aliases: &[],
        description: "decimation factor (1 = bypass)",
        default: "0.0",
        min: 0.0,
        max: 128.0,
    }],
};

/// Sample-and-hold decimator for lo-fi effects.
///
/// Holds input values for `factor` samples, reducing effective sample rate.
/// Often combined with bit depth reduction for full bitcrusher effects.
#[derive(Clone, Copy, Default)]
pub struct Coarse {
    /// Currently held sample value.
    hold: f32,
    /// Sample counter (0 to factor-1).
    t: usize,
}

impl Coarse {
    /// Block-rate variant: processes `n` samples of stereo-frame buffer in place
    /// on channel `ch`. `factor.max(1.0) as usize` hoists to block entry.
    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ch: usize, factor: f32) {
        let stride = factor.max(1.0) as usize;
        for slot in buf.iter_mut().take(n) {
            if self.t == 0 {
                self.hold = slot[ch];
            }
            self.t = (self.t + 1) % stride;
            slot[ch] = self.hold;
        }
    }
}
