//! Synthesized drum sources.
//!
//! Each drum generates a shaped waveform with internal timbral dynamics.
//! The engine's existing ADSR controls overall amplitude.

use std::f32::consts::TAU;

use crate::dsp::{exp2f, expf, sinf, SvfMode};
use crate::types::Source;

use super::Voice;

const COWBELL_RATIO: f32 = 1.4836;
const CYMBAL_RATIOS: [f32; 6] = [1.0, 1.3394, 1.7726, 2.1753, 2.6651, 3.1467];

/// Drum oscillator with waveform morphing.
/// waveform: 0.0 = sine, 0.5 = triangle, 1.0 = sawtooth
#[inline]
fn drum_osc(phase: f32, waveform: f32) -> f32 {
    if waveform <= 0.0 {
        return sinf(phase * TAU);
    }
    if waveform >= 1.0 {
        return phase * 2.0 - 1.0;
    }
    let tri = if phase < 0.5 { 4.0 * phase - 1.0 } else { 3.0 - 4.0 * phase };
    if waveform >= 0.5 {
        let t = (waveform - 0.5) * 2.0;
        let saw = phase * 2.0 - 1.0;
        tri + t * (saw - tri)
    } else {
        let t = waveform * 2.0;
        let sine = sinf(phase * TAU);
        sine + t * (tri - sine)
    }
}

impl Voice {
    #[inline]
    pub(super) fn run_drum(&mut self, freq: f32, isr: f32) {
        self.ch[0] = match self.params.sound {
            Source::Kick => self.drum_kick(freq, isr),
            Source::Snare => self.drum_snare(freq, isr),
            Source::Hat => self.drum_hat(freq, isr),
            Source::Tom => self.drum_tom(freq, isr),

            Source::Rim => self.drum_rim(freq, isr),
            Source::Cowbell => self.drum_cowbell(freq, isr),
            Source::Cymbal => self.drum_cymbal(freq, isr),
            _ => 0.0,
        } * 0.2;
    }

    #[inline]
    fn drum_kick(&mut self, freq: f32, isr: f32) -> f32 {
        let sweep_oct = self.params.morph * 4.0;
        let rate = 20.0 + self.params.harmonics * 80.0;
        let pitch_env = expf(-self.time * rate);
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        let phase = self.phasor.phase;
        let sample = drum_osc(phase, self.params.wave);
        self.phasor.phase = (phase + actual_freq * isr).fract();

        let drive = self.params.timbre * 4.0;
        if drive > 0.0 {
            let x = sample * (1.0 + drive);
            x / (1.0 + x.abs())
        } else {
            sample
        }
    }

    #[inline]
    fn drum_snare(&mut self, freq: f32, isr: f32) -> f32 {
        let rate = 40.0 + self.params.harmonics * 60.0;
        let pitch_env = expf(-self.time * rate);
        let actual_freq = freq * exp2f(1.5 * pitch_env);

        let phase = self.phasor.phase;
        let body = drum_osc(phase, self.params.wave);
        self.phasor.phase = (phase + actual_freq * isr).fract();

        let noise = self.white();
        let brightness = 2000.0 + self.params.harmonics * 6000.0;
        self.drum_svf.cutoff = brightness;
        let filtered_noise = self.drum_svf.process(noise, SvfMode::Bp, 0.3, self.sr);

        let mix = self.params.timbre;
        body * (1.0 - mix) + filtered_noise * mix * 2.0
    }

    #[inline]
    fn drum_hat(&mut self, freq: f32, isr: f32) -> f32 {
        let mod_depth = 0.5 + self.params.morph * 2.5;

        let m2 = self.white();

        let p1 = &mut self.spread_phasors[0];
        let m1 = sinf((p1.phase + mod_depth * m2) * TAU);
        p1.phase = (p1.phase + 2.0 * freq * isr).fract();

        let m0 = sinf((self.phasor.phase + mod_depth * m1) * TAU);
        self.phasor.phase = (self.phasor.phase + freq * isr).fract();

        let tone = 800.0 + self.params.harmonics * 17200.0;
        let q = 0.05 + self.params.timbre * 0.9;
        self.drum_svf.cutoff = tone;
        self.drum_svf.process(m0, SvfMode::Lp, q, self.sr)
    }

    #[inline]
    fn drum_tom(&mut self, freq: f32, isr: f32) -> f32 {
        let sweep_oct = self.params.morph * 1.5;
        let rate = 15.0 + self.params.harmonics * 40.0;
        let pitch_env = expf(-self.time * rate);
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        let phase = self.phasor.phase;
        let body = drum_osc(phase, self.params.wave);
        self.phasor.phase = (phase + actual_freq * isr).fract();

        let noise = self.white();
        let mix = self.params.timbre * 0.3;
        body * (1.0 - mix) + noise * mix
    }

    #[inline]
    fn drum_rim(&mut self, freq: f32, isr: f32) -> f32 {
        let sweep_oct = self.params.morph * 2.0;
        let pitch_env = expf(-self.time * 200.0);
        let actual_freq = freq * exp2f(sweep_oct * pitch_env);

        let phase = self.phasor.phase;
        let body = drum_osc(phase, self.params.wave);
        self.phasor.phase = (phase + actual_freq * isr).fract();

        let noise = self.white();
        let brightness = 3000.0 + self.params.harmonics * 8000.0;
        self.drum_svf.cutoff = brightness;
        let filtered_noise = self.drum_svf.process(noise, SvfMode::Bp, 0.5, self.sr);

        let mix = self.params.timbre;
        body * (1.0 - mix) + filtered_noise * mix * 2.0
    }

    #[inline]
    fn drum_cowbell(&mut self, freq: f32, isr: f32) -> f32 {
        let detune = 1.0 + (COWBELL_RATIO - 1.0) * (0.5 + self.params.morph * 0.5);
        let freq2 = freq * detune;

        let p0 = &mut self.spread_phasors[0];
        let sq0 = if p0.phase < 0.5 { 1.0 } else { -1.0 };
        p0.phase = (p0.phase + freq * isr).fract();

        let p1 = &mut self.spread_phasors[1];
        let sq1 = if p1.phase < 0.5 { 1.0 } else { -1.0 };
        p1.phase = (p1.phase + freq2 * isr).fract();

        let mixed = (sq0 + sq1) * 0.5;

        // Soft saturation — emulates 808's "swing type" VCAs
        let drive = 1.0 + self.params.timbre * 4.0;
        let driven = mixed * drive;
        let saturated = driven / (1.0 + driven.abs());

        let cutoff = freq2 * (1.1 + self.params.harmonics * 3.0);
        self.drum_svf.cutoff = cutoff;
        self.drum_svf.process(saturated, SvfMode::Bp, 0.47, self.sr)
    }

    #[inline]
    fn drum_cymbal(&mut self, freq: f32, isr: f32) -> f32 {
        let spread_amt = 0.5 + self.params.morph * 1.5;

        let mut metallic = 0.0_f32;
        for (i, &ratio) in CYMBAL_RATIOS.iter().enumerate() {
            let r = 1.0 + (ratio - 1.0) * spread_amt;
            let cym_freq = freq * r;
            let p = &mut self.spread_phasors[i];
            let pulse = if p.phase < 0.5 { 1.0 } else { -1.0 };
            p.phase = (p.phase + cym_freq * isr).fract();
            metallic += pulse;
        }
        metallic /= 6.0;

        let noise = self.white() * self.params.timbre;
        let combined = metallic + noise;

        let cutoff = 2500.0 + self.params.harmonics * 12000.0;
        self.drum_svf.cutoff = cutoff;
        self.drum_svf.process(combined, SvfMode::Hp, 0.15, self.sr)
    }
}
