// Moog ladder filter based on "An Improved Virtual Analog Model of the Moog Ladder Filter"
// by Stefano D'Angelo and Vesa Välimäki. Multimode output (LP/HP/BP) via stage-tap coefficient mixing.

use crate::dsp::fast_tanh_f32;
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo};
use std::f32::consts::PI;

pub const INFO_LLPF: ModuleInfo = ModuleInfo {
    name: "llpf",
    description: "Moog-style ladder lowpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "llpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "llpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

pub const INFO_LHPF: ModuleInfo = ModuleInfo {
    name: "lhpf",
    description: "Moog-style ladder highpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "lhpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "lhpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

pub const INFO_LBPF: ModuleInfo = ModuleInfo {
    name: "lbpf",
    description: "Moog-style ladder bandpass filter",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "lbpf",
            aliases: &[],
            description: "cutoff frequency in Hz",
            default: "0.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "lbpq",
            aliases: &[],
            description: "resonance",
            default: "0.2",
            min: 0.0,
            max: 1.0,
        },
    ],
};

#[derive(Clone, Copy, PartialEq)]
pub enum LadderMode {
    Lp,
    Hp,
    Bp,
}

const VT: f32 = 0.312;
const VT2: f32 = 2.0 * VT;

#[derive(Clone, Copy)]
pub struct LadderFilter {
    v: [f32; 4],
    dv: [f32; 4],
    tv: [f32; 4],
    cached_cutoff: f32,
    cached_g: f32,
    cached_inv_2sr: f32,
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self {
            v: [0.0; 4],
            dv: [0.0; 4],
            tv: [0.0; 4],
            cached_cutoff: 0.0,
            cached_g: 0.0,
            cached_inv_2sr: 0.0,
        }
    }
}

impl LadderFilter {
    #[inline]
    pub fn process(
        &mut self,
        input: f32,
        cutoff: f32,
        resonance: f32,
        mode: LadderMode,
        sr: f32,
    ) -> f32 {
        let cutoff = cutoff.clamp(20.0, sr * 0.45);
        let cutoff_delta = (cutoff - self.cached_cutoff).abs() / self.cached_cutoff.max(1.0);
        if cutoff_delta > 0.001 || self.cached_inv_2sr == 0.0 {
            self.cached_cutoff = cutoff;
            let x = (PI * cutoff) / sr;
            self.cached_g = 4.0 * PI * VT * cutoff * (1.0 - x) / (1.0 + x);
            self.cached_inv_2sr = 0.5 / sr;
        }
        let g = self.cached_g;
        let res = resonance.clamp(0.0, 1.0) * 4.0;
        let inv_2sr = self.cached_inv_2sr;

        let dv0 = -g * (fast_tanh_f32((input + res * self.v[3]) / VT2) + self.tv[0]);
        self.v[0] += (dv0 + self.dv[0]) * inv_2sr;
        self.dv[0] = dv0;
        self.tv[0] = fast_tanh_f32(self.v[0] / VT2);

        let dv1 = g * (self.tv[0] - self.tv[1]);
        self.v[1] += (dv1 + self.dv[1]) * inv_2sr;
        self.dv[1] = dv1;
        self.tv[1] = fast_tanh_f32(self.v[1] / VT2);

        let dv2 = g * (self.tv[1] - self.tv[2]);
        self.v[2] += (dv2 + self.dv[2]) * inv_2sr;
        self.dv[2] = dv2;
        self.tv[2] = fast_tanh_f32(self.v[2] / VT2);

        let dv3 = g * (self.tv[2] - self.tv[3]);
        self.v[3] += (dv3 + self.dv[3]) * inv_2sr;
        self.dv[3] = dv3;
        self.tv[3] = fast_tanh_f32(self.v[3] / VT2);

        match mode {
            LadderMode::Lp => self.v[3],
            LadderMode::Hp => {
                input - 4.0 * self.v[0] + 6.0 * self.v[1] - 4.0 * self.v[2] + self.v[3]
            }
            LadderMode::Bp => 4.0 * self.v[1] - 8.0 * self.v[2] + 4.0 * self.v[3],
        }
    }
}
