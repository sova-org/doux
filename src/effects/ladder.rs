// Moog ladder filter based on "An Improved Virtual Analog Model of the Moog Ladder Filter"
// by Stefano D'Angelo and Vesa Välimäki. Multimode output (LP/HP/BP) via stage-tap coefficient mixing.

use crate::dsp::fast_tanh;
use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum LadderMode {
    Lp,
    Hp,
    Bp,
}

const VT: f64 = 0.312;
const VT2: f64 = 2.0 * VT;

#[derive(Clone, Copy)]
pub struct LadderFilter {
    v: [f64; 4],
    dv: [f64; 4],
    tv: [f64; 4],
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self {
            v: [0.0; 4],
            dv: [0.0; 4],
            tv: [0.0; 4],
        }
    }
}

impl LadderFilter {
    pub fn process(
        &mut self,
        input: f32,
        cutoff: f32,
        resonance: f32,
        mode: LadderMode,
        sr: f32,
    ) -> f32 {
        let cutoff = cutoff.clamp(20.0, sr * 0.45);
        let x = (PI * cutoff as f64) / sr as f64;
        let g = 4.0 * PI * VT * cutoff as f64 * (1.0 - x) / (1.0 + x);

        let res = (resonance * 4.0) as f64;
        let sr64 = sr as f64;
        let inp = input as f64;

        let dv0 = -g * (fast_tanh((inp + res * self.v[3]) / VT2) + self.tv[0]);
        self.v[0] += (dv0 + self.dv[0]) / (2.0 * sr64);
        self.dv[0] = dv0;
        self.tv[0] = fast_tanh(self.v[0] / VT2);

        let dv1 = g * (self.tv[0] - self.tv[1]);
        self.v[1] += (dv1 + self.dv[1]) / (2.0 * sr64);
        self.dv[1] = dv1;
        self.tv[1] = fast_tanh(self.v[1] / VT2);

        let dv2 = g * (self.tv[1] - self.tv[2]);
        self.v[2] += (dv2 + self.dv[2]) / (2.0 * sr64);
        self.dv[2] = dv2;
        self.tv[2] = fast_tanh(self.v[2] / VT2);

        let dv3 = g * (self.tv[2] - self.tv[3]);
        self.v[3] += (dv3 + self.dv[3]) / (2.0 * sr64);
        self.dv[3] = dv3;
        self.tv[3] = fast_tanh(self.v[3] / VT2);

        let out = match mode {
            LadderMode::Lp => self.v[3],
            LadderMode::Hp => inp - 4.0 * self.v[0] + 6.0 * self.v[1] - 4.0 * self.v[2] + self.v[3],
            LadderMode::Bp => 4.0 * self.v[1] - 8.0 * self.v[2] + 4.0 * self.v[3],
        };
        out as f32
    }
}
