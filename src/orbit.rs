use crate::dsp::Phasor;
use crate::effects::{Comb, DattorroVerb, Delay, DelayParams, Feedback, VitalVerb};
use crate::types::{DelayType, LfoShape, ReverbType, CHANNELS};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF: u32 = 48000;

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
    pub fb_lfo: f32,
    pub fb_lfo_depth: f32,
    pub fb_lfo_shape: LfoShape,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            delay_time: 0.333,
            delay_feedback: 0.6,
            delay_type: DelayType::Standard,
            verb_type: ReverbType::Space,
            verb_decay: 0.75,
            verb_damp: 0.95,
            verb_predelay: 0.1,
            verb_diff: 0.7,
            verb_prelow: 0.2,
            verb_prehigh: 0.8,
            verb_lowcut: 0.5,
            verb_highcut: 0.7,
            verb_lowgain: 0.4,
            verb_chorus: 0.3,
            verb_chorus_freq: 0.2,
            comb_freq: 220.0,
            comb_feedback: 0.9,
            comb_damp: 0.1,
            fb_time: 10.0,
            fb_damp: 0.0,
            fb_lfo: 0.0,
            fb_lfo_depth: 0.5,
            fb_lfo_shape: LfoShape::Sine,
        }
    }
}

pub struct Orbit {
    pub delay: Delay,
    pub delay_send: [f32; CHANNELS],
    pub delay_out: [f32; CHANNELS],
    pub verb: DattorroVerb,
    pub vital: VitalVerb,
    pub verb_send: [f32; CHANNELS],
    pub verb_out: [f32; CHANNELS],
    pub comb: Comb,
    pub comb_send: [f32; CHANNELS],
    pub comb_out: [f32; CHANNELS],
    pub fb: Feedback,
    pub fb_phasor: Phasor,
    pub fb_send: [f32; CHANNELS],
    pub fb_level: f32,
    pub fb_out: [f32; CHANNELS],
    pub params: EffectParams,
    pub sr: f32,
    silent_samples: u32,
}

impl Orbit {
    pub fn new(sr: f32) -> Self {
        Self {
            delay: Delay::new(),
            delay_send: [0.0; CHANNELS],
            delay_out: [0.0; CHANNELS],
            verb: DattorroVerb::new(sr),
            vital: VitalVerb::new(sr),
            verb_send: [0.0; CHANNELS],
            verb_out: [0.0; CHANNELS],
            comb: Comb::default(),
            comb_send: [0.0; CHANNELS],
            comb_out: [0.0; CHANNELS],
            fb: Feedback::default(),
            fb_phasor: Phasor::default(),
            fb_send: [0.0; CHANNELS],
            fb_level: 0.0,
            fb_out: [0.0; CHANNELS],
            params: EffectParams::default(),
            sr,
            silent_samples: SILENCE_HOLDOFF + 1,
        }
    }

    pub fn clear_sends(&mut self) {
        self.delay_send = [0.0; CHANNELS];
        self.verb_send = [0.0; CHANNELS];
        self.comb_send = [0.0; CHANNELS];
        self.fb_send = [0.0; CHANNELS];
        self.fb_level = 0.0;
    }

    pub fn add_delay_send(&mut self, ch: usize, value: f32) {
        self.delay_send[ch] += value;
    }

    pub fn add_verb_send(&mut self, ch: usize, value: f32) {
        self.verb_send[ch] += value;
    }

    pub fn add_comb_send(&mut self, ch: usize, value: f32) {
        self.comb_send[ch] += value;
    }

    pub fn add_fb_send(&mut self, ch: usize, value: f32) {
        self.fb_send[ch] += value;
    }

    pub fn process(&mut self) {
        let p = &self.params;
        let has_input = self.delay_send[0] != 0.0
            || self.delay_send[1] != 0.0
            || self.verb_send[0] != 0.0
            || self.verb_send[1] != 0.0
            || self.comb_send[0] != 0.0
            || self.comb_send[1] != 0.0
            || self.fb_send[0] != 0.0
            || self.fb_send[1] != 0.0;

        if has_input {
            self.silent_samples = 0;
        } else if self.silent_samples > SILENCE_HOLDOFF {
            self.delay_out = [0.0; CHANNELS];
            self.verb_out = [0.0; CHANNELS];
            self.comb_out = [0.0; CHANNELS];
            self.fb_out = [0.0; CHANNELS];
            return;
        }

        self.delay_out = self.delay.process(
            self.delay_send,
            &DelayParams {
                time: p.delay_time,
                feedback: p.delay_feedback,
                delay_type: p.delay_type,
                sr: self.sr,
            },
        );

        let verb_input = (self.verb_send[0] + self.verb_send[1]) * 0.5;
        self.verb_out = match p.verb_type {
            ReverbType::Plate => self.verb.process(
                verb_input,
                p.verb_decay,
                p.verb_damp,
                p.verb_predelay,
                p.verb_diff,
            ),
            ReverbType::Space => self.vital.process(
                verb_input,
                p.verb_decay,
                p.verb_damp,
                p.verb_predelay,
                p.verb_diff,
                p.verb_prelow,
                p.verb_prehigh,
                p.verb_lowcut,
                p.verb_highcut,
                p.verb_lowgain,
                p.verb_chorus,
                p.verb_chorus_freq,
            ),
        };

        let comb_input = (self.comb_send[0] + self.comb_send[1]) * 0.5;
        let comb_out = self.comb.process(
            comb_input,
            p.comb_freq,
            p.comb_feedback,
            p.comb_damp,
            self.sr,
        );
        self.comb_out = [comb_out, comb_out];

        let fb_input = (self.fb_send[0] + self.fb_send[1]) * 0.5;
        let isr = 1.0 / self.sr;
        let fb_time = if p.fb_lfo > 0.0 {
            let lfo = self.fb_phasor.lfo(p.fb_lfo_shape, p.fb_lfo, isr);
            p.fb_time + lfo * p.fb_lfo_depth * p.fb_time * 0.5
        } else {
            p.fb_time
        };
        let fb_out = self.fb.process(fb_input, self.fb_level, fb_time, p.fb_damp, self.sr);
        self.fb_out = [fb_out, fb_out];

        let energy = self.delay_out[0].abs()
            + self.delay_out[1].abs()
            + self.verb_out[0].abs()
            + self.verb_out[1].abs()
            + self.comb_out[0].abs()
            + self.comb_out[1].abs()
            + self.fb_out[0].abs()
            + self.fb_out[1].abs();

        if energy < SILENCE_THRESHOLD {
            self.silent_samples = self.silent_samples.saturating_add(1);
        } else {
            self.silent_samples = 0;
        }
    }
}
