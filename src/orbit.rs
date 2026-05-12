use crate::dsp::Phasor;
use crate::effects::{Comb, Compressor, DattorroVerb, Delay, DelayParams, Feedback, VitalVerb};
use crate::types::{DelayType, LfoShape, ReverbType, CHANNELS};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF_SECS: f32 = 1.0;

#[derive(Clone, Copy)]
pub struct EffectParams {
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_type: DelayType,
    pub verb_type: ReverbType,
    pub verb_decay: f32,
    pub verb_damp: f32,
    pub verb_predelay: f32,
    pub verb_diff: f32,
    pub verb_size: f32,
    pub verb_prelow: f32,
    pub verb_prehigh: f32,
    pub verb_lowcut: f32,
    pub verb_highcut: f32,
    pub verb_lowgain: f32,
    pub verb_chorus: f32,
    pub verb_chorus_freq: f32,
    pub comb_freq: f32,
    pub comb_feedback: f32,
    pub comb_damp: f32,
    pub fb_time: f32,
    pub fb_damp: f32,
    pub fb_cross: f32,
    pub fb_lfo: f32,
    pub fb_lfo_depth: f32,
    pub fb_lfo_shape: LfoShape,
    pub comp: f32,
    pub comp_attack: f32,
    pub comp_release: f32,
    pub comp_orbit: usize,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            delay_time: 0.333,
            delay_feedback: 0.6,
            delay_type: DelayType::Standard,
            verb_type: ReverbType::Space,
            verb_decay: 0.55,
            verb_damp: 0.7,
            verb_predelay: 0.0,
            verb_diff: 0.6,
            verb_size: 0.75,
            verb_prelow: 0.2,
            verb_prehigh: 0.9,
            verb_lowcut: 0.5,
            verb_highcut: 0.7,
            verb_lowgain: 0.1,
            verb_chorus: 0.3,
            verb_chorus_freq: 0.65,
            comb_freq: 220.0,
            comb_feedback: 0.9,
            comb_damp: 0.1,
            fb_time: 10.0,
            fb_damp: 0.0,
            fb_cross: 0.0,
            fb_lfo: 0.0,
            fb_lfo_depth: 0.5,
            fb_lfo_shape: LfoShape::Sine,
            comp: 0.0,
            comp_attack: 0.01,
            comp_release: 0.15,
            comp_orbit: 0,
        }
    }
}

// SuperDirt-style chain: voices accumulate into `bus`; each FX reads
// `bus * send_level`, adds its wet back into `bus`, in order. Order matters —
// later FX see the running signal including previous FX wet.
//
// Chain order: comb → fb → delay → verb. Tonal/short → spatial/long.
// Reverb last so it captures delay echoes (the load-bearing reason for chaining).
pub struct Orbit {
    pub bus: [f32; CHANNELS],
    pub delay: Delay,
    pub delay_level: f32,
    pub verb: [DattorroVerb; CHANNELS],
    pub vital: VitalVerb,
    pub verb_level: f32,
    pub comb: [Comb; CHANNELS],
    pub comb_level: f32,
    pub fb: Feedback,
    pub fb_phasor: Phasor,
    pub fb_level: f32,
    pub comp: Compressor,
    pub params: EffectParams,
    pub sr: f32,
    silent_samples: u32,
    silence_holdoff: u32,
}

impl Orbit {
    pub fn new(sr: f32) -> Self {
        let silence_holdoff = (sr * SILENCE_HOLDOFF_SECS) as u32;
        Self {
            bus: [0.0; CHANNELS],
            delay: Delay::default(),
            delay_level: 0.0,
            verb: std::array::from_fn(|_| DattorroVerb::new(sr)),
            vital: VitalVerb::new(sr),
            verb_level: 0.0,
            comb: [Comb::default(); CHANNELS],
            comb_level: 0.0,
            fb: Feedback::default(),
            fb_phasor: Phasor::default(),
            fb_level: 0.0,
            comp: Compressor::default(),
            params: EffectParams::default(),
            sr,
            silent_samples: silence_holdoff + 1,
            silence_holdoff,
        }
    }

    pub fn clear_bus(&mut self) {
        self.bus = [0.0; CHANNELS];
    }

    pub fn add_dry(&mut self, ch: usize, value: f32) {
        self.bus[ch] += value;
    }

    pub fn process(&mut self) {
        let p = &self.params;
        let has_input = self.bus[0] != 0.0 || self.bus[1] != 0.0;

        if has_input {
            self.silent_samples = 0;
        } else if self.silent_samples > self.silence_holdoff {
            return;
        }

        // Comb (per-channel mono resonator)
        if self.comb_level > 0.0 {
            let mut wet = [0.0_f32; CHANNELS];
            for (channel, w) in wet.iter_mut().enumerate() {
                *w = self.comb[channel].process(
                    self.bus[channel] * self.comb_level,
                    p.comb_freq,
                    p.comb_feedback,
                    p.comb_damp,
                    self.sr,
                );
            }
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Feedback (stereo short delay with cross-channel)
        if self.fb_level > 0.0 {
            let isr = 1.0 / self.sr;
            let fb_time = if p.fb_lfo > 0.0 {
                let lfo = self.fb_phasor.lfo(p.fb_lfo_shape, p.fb_lfo, isr);
                p.fb_time + lfo * p.fb_lfo_depth * p.fb_time * 0.5
            } else {
                p.fb_time
            };
            let fb_in = [self.bus[0] * self.fb_level, self.bus[1] * self.fb_level];
            let wet = self.fb.process(
                fb_in,
                self.fb_level,
                fb_time,
                p.fb_damp,
                p.fb_cross,
                self.sr,
            );
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Delay (stereo)
        if self.delay_level > 0.0 {
            let delay_in = [
                self.bus[0] * self.delay_level,
                self.bus[1] * self.delay_level,
            ];
            let wet = self.delay.process(
                delay_in,
                &DelayParams {
                    time: p.delay_time,
                    feedback: p.delay_feedback,
                    delay_type: p.delay_type,
                    sr: self.sr,
                },
            );
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Reverb — last in chain so it captures delay echoes
        if self.verb_level > 0.0 {
            let verb_in = [self.bus[0] * self.verb_level, self.bus[1] * self.verb_level];
            let wet = match p.verb_type {
                ReverbType::Plate => {
                    let mut out = [0.0; CHANNELS];
                    for (channel, vin) in verb_in.iter().enumerate() {
                        let w = self.verb[channel].process(
                            *vin,
                            p.verb_decay,
                            p.verb_damp,
                            p.verb_predelay,
                            p.verb_diff,
                        );
                        out[0] += w[0];
                        out[1] += w[1];
                    }
                    out
                }
                ReverbType::Space => self.vital.process(
                    verb_in,
                    p.verb_decay,
                    p.verb_damp,
                    p.verb_predelay,
                    p.verb_size,
                    p.verb_prelow,
                    p.verb_prehigh,
                    p.verb_lowcut,
                    p.verb_highcut,
                    p.verb_lowgain,
                    p.verb_chorus,
                    p.verb_chorus_freq,
                    p.verb_diff,
                ),
            };
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        let energy = self.bus[0].abs() + self.bus[1].abs();
        if energy < SILENCE_THRESHOLD {
            self.silent_samples = self.silent_samples.saturating_add(1);
        } else {
            self.silent_samples = 0;
        }
    }
}
