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

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

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

// The sample-rate decimator is now Faust-generated; see [`super::faust_dsp`].
// Only the `coarse` parameter metadata (`INFO`) remains here.
