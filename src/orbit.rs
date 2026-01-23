use crate::effects::{Comb, DattorroVerb, Delay, DelayParams, FdnVerb};
use crate::types::{DelayType, ReverbType, CHANNELS};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF: u32 = 48000;

#[derive(Clone, Copy, Default)]
pub struct EffectParams {
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_type: DelayType,
    pub verb_type: ReverbType,
    pub verb_decay: f32,
    pub verb_damp: f32,
    pub verb_predelay: f32,
    pub verb_diff: f32,
    pub comb_freq: f32,
    pub comb_feedback: f32,
    pub comb_damp: f32,
}

pub struct Orbit {
    pub delay: Delay,
    pub delay_send: [f32; CHANNELS],
    pub delay_out: [f32; CHANNELS],
    pub verb: DattorroVerb,
    pub fdn: FdnVerb,
    pub verb_send: [f32; CHANNELS],
    pub verb_out: [f32; CHANNELS],
    pub comb: Comb,
    pub comb_send: [f32; CHANNELS],
    pub comb_out: [f32; CHANNELS],
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
            fdn: FdnVerb::new(sr),
            verb_send: [0.0; CHANNELS],
            verb_out: [0.0; CHANNELS],
            comb: Comb::default(),
            comb_send: [0.0; CHANNELS],
            comb_out: [0.0; CHANNELS],
            sr,
            silent_samples: SILENCE_HOLDOFF + 1,
        }
    }

    pub fn clear_sends(&mut self) {
        self.delay_send = [0.0; CHANNELS];
        self.verb_send = [0.0; CHANNELS];
        self.comb_send = [0.0; CHANNELS];
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

    pub fn process(&mut self, p: &EffectParams) {
        let has_input = self.delay_send[0] != 0.0
            || self.delay_send[1] != 0.0
            || self.verb_send[0] != 0.0
            || self.verb_send[1] != 0.0
            || self.comb_send[0] != 0.0
            || self.comb_send[1] != 0.0;

        if has_input {
            self.silent_samples = 0;
        } else if self.silent_samples > SILENCE_HOLDOFF {
            self.delay_out = [0.0; CHANNELS];
            self.verb_out = [0.0; CHANNELS];
            self.comb_out = [0.0; CHANNELS];
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
            ReverbType::Dattorro => self.verb.process(
                verb_input,
                p.verb_decay,
                p.verb_damp,
                p.verb_predelay,
                p.verb_diff,
            ),
            ReverbType::Fdn => self.fdn.process(
                verb_input,
                p.verb_decay,
                p.verb_damp,
                p.verb_diff, // size
                p.verb_predelay, // modulation
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

        let energy = self.delay_out[0].abs()
            + self.delay_out[1].abs()
            + self.verb_out[0].abs()
            + self.verb_out[1].abs()
            + self.comb_out[0].abs()
            + self.comb_out[1].abs();

        if energy < SILENCE_THRESHOLD {
            self.silent_samples = self.silent_samples.saturating_add(1);
        } else {
            self.silent_samples = 0;
        }
    }
}
