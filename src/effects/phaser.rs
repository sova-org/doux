//! Phaser effect: a cascade of first-order allpass stages with feedback.
//!
//! The classic phaser sound comes from summing the dry signal with an
//! allpass-shifted copy of itself. Where the cascade's phase response crosses
//! 180°, dry and wet cancel and a notch forms; the notches sweep as an LFO
//! modulates the shared allpass break frequency. Feedback emphasises the
//! resonant peaks between notches for a throatier, more analog character.
//!
//! # Signal Flow
//!
//! ```text
//! in ──┬─────────────────────────────┐
//!      │                             ▼
//!      │   ┌─ AP×NUM_STAGES (shared fc) ─┐   (+) ── out
//!      └──▶│  fc swept per-sample by LFO  │──┬──▲ dry + wet
//!       ▲  └──────────────────────────────┘  │
//!       └────────── feedback (k) ◀────────────┘
//! ```

use crate::dsp::{exp2f, fast_tan, ftz, Phasor};
use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};
use std::f32::consts::PI;

pub const INFO: ModuleInfo = ModuleInfo {
    name: "phaser",
    description: "6-stage allpass phaser with feedback resonance",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "phaser",
            aliases: &["phaserrate"],
            description: "LFO rate in Hz (0 = bypass)",
            default: "0.0",
            min: 0.0,
            max: 100.0,
        },
        ParamInfo {
            name: "phaserdepth",
            aliases: &[],
            description: "feedback resonance",
            default: "0.75",
            min: 0.0,
            max: 0.95,
        },
        ParamInfo {
            name: "phasersweep",
            aliases: &[],
            description: "modulation range in cents",
            default: "1200.0",
            min: 0.0,
            max: 20000.0,
        },
        ParamInfo {
            name: "phasercenter",
            aliases: &[],
            description: "base center frequency in Hz",
            default: "800.0",
            min: 0.0,
            max: 20000.0,
        },
    ],
};

/// Number of cascaded first-order allpass stages. Each pair of stages yields
/// one swept notch, so 6 stages give the classic three-notch voicing.
const NUM_STAGES: usize = 6;

/// First-order allpass section, `H(z) = (a + z⁻¹) / (1 + a·z⁻¹)`.
#[derive(Clone, Copy, Default)]
struct Allpass1 {
    x1: f32,
    y1: f32,
}

impl Allpass1 {
    #[inline]
    fn process(&mut self, x: f32, a: f32) -> f32 {
        let y = a * x + self.x1 - a * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

/// Multi-stage allpass phaser with LFO-swept break frequency and feedback.
#[derive(Clone, Copy, Default)]
pub struct Phaser {
    stages: [Allpass1; NUM_STAGES],
    fb_state: f32,
    lfo: Phasor,
}

impl Phaser {
    /// Builds a phaser seeded for stereo width. The right channel's LFO is
    /// offset a quarter cycle (90°) so the notches sweep out of phase between
    /// channels, widening the image even for mono sources.
    pub fn new(ch: usize) -> Self {
        let mut p = Self::default();
        p.lfo.phase = if ch == 1 { 0.25 } else { 0.0 };
        p
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        input: f32,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
        isr: f32,
    ) -> f32 {
        let k = (depth * 0.9).clamp(0.0, 0.9);
        let max_fc = sr * 0.45;

        let lfo = self.lfo.sine(rate, isr);
        let fc = (center * exp2f(lfo * sweep * (1.0 / 1200.0))).clamp(20.0, max_fc);
        let t = (PI * fc / sr).min(PI * 0.4999);
        let tan_t = fast_tan(t);
        let a = (tan_t - 1.0) / (tan_t + 1.0);

        let mut wet = input + ftz(self.fb_state, 1e-20) * k;
        for stage in &mut self.stages {
            wet = stage.process(wet, a);
        }
        self.fb_state = wet;

        0.5 * input + 0.5 * wet
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
        isr: f32,
    ) {
        let k = (depth * 0.9).clamp(0.0, 0.9);
        let max_fc = sr * 0.45;
        for slot in buf.iter_mut().take(n) {
            let lfo = self.lfo.sine(rate, isr);
            let fc = (center * exp2f(lfo * sweep * (1.0 / 1200.0))).clamp(20.0, max_fc);
            let t = (PI * fc / sr).min(PI * 0.4999);
            let tan_t = fast_tan(t);
            let a = (tan_t - 1.0) / (tan_t + 1.0);

            let input = slot[ch];
            let mut wet = input + ftz(self.fb_state, 1e-20) * k;
            for stage in &mut self.stages {
                wet = stage.process(wet, a);
            }
            self.fb_state = wet;

            slot[ch] = 0.5 * input + 0.5 * wet;
        }
    }
}
