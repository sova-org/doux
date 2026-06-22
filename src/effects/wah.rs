//! Envelope-follower auto-wah — Faust-generated; see [`super::faust_dsp`].
//! Only the parameter metadata (`INFO`) lives here.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "wah",
    description: "Auto-wah: resonant bandpass swept by the input envelope",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "wah",
            aliases: &[],
            description: "dry/wet mix (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "wahpeak",
            aliases: &[],
            description: "resonance / peak",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "wahsens",
            aliases: &[],
            description: "envelope sensitivity",
            default: "0.5",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "wahmanual",
            aliases: &[],
            description: "base cutoff in Hz (resting position)",
            default: "400.0",
            min: 100.0,
            max: 4000.0,
        },
    ],
};
