// Moog ladder filter based on "An Improved Virtual Analog Model of the Moog Ladder Filter"
// by Stefano D'Angelo and Vesa Välimäki. Multimode output (LP/HP/BP) via stage-tap coefficient mixing.

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
    g: f64,
    cached_cutoff: f32,
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self {
            v: [0.0; 4],
            dv: [0.0; 4],
            tv: [0.0; 4],
            g: 0.0,
            cached_cutoff: 0.0,
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
        if (cutoff - self.cached_cutoff).abs() > 0.1 {
            let x = (PI * cutoff as f64) / sr as f64;
            self.g = 4.0 * PI * VT * cutoff as f64 * (1.0 - x) / (1.0 + x);
            self.cached_cutoff = cutoff;
        }

        let res = (resonance * 4.0) as f64;
        let sr64 = sr as f64;
        let inp = input as f64;

        let dv0 = -self.g * (((inp + res * self.v[3]) / VT2).tanh() + self.tv[0]);
        self.v[0] += (dv0 + self.dv[0]) / (2.0 * sr64);
        self.dv[0] = dv0;
        self.tv[0] = (self.v[0] / VT2).tanh();

        let dv1 = self.g * (self.tv[0] - self.tv[1]);
        self.v[1] += (dv1 + self.dv[1]) / (2.0 * sr64);
        self.dv[1] = dv1;
        self.tv[1] = (self.v[1] / VT2).tanh();

        let dv2 = self.g * (self.tv[1] - self.tv[2]);
        self.v[2] += (dv2 + self.dv[2]) / (2.0 * sr64);
        self.dv[2] = dv2;
        self.tv[2] = (self.v[2] / VT2).tanh();

        let dv3 = self.g * (self.tv[2] - self.tv[3]);
        self.v[3] += (dv3 + self.dv[3]) / (2.0 * sr64);
        self.dv[3] = dv3;
        self.tv[3] = (self.v[3] / VT2).tanh();

        let out = match mode {
            LadderMode::Lp => self.v[3],
            LadderMode::Hp => inp - 4.0 * self.v[0] + 6.0 * self.v[1] - 4.0 * self.v[2] + self.v[3],
            LadderMode::Bp => 4.0 * self.v[1] - 8.0 * self.v[2] + 4.0 * self.v[3],
        };
        out as f32
    }
}
