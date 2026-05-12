use crate::effects::{
    Comb, CombParams, Compressor, DattorroVerb, Delay, Feedback, ReverbParams, VitalVerb,
};
use crate::types::{ReverbType, CHANNELS};

const SILENCE_THRESHOLD: f32 = 1e-7;
const SILENCE_HOLDOFF_SECS: f32 = 1.0;

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
    pub dattorro: [DattorroVerb; CHANNELS],
    pub vital: VitalVerb,
    pub reverb_params: ReverbParams,
    pub verb_level: f32,
    pub comb: [Comb; CHANNELS],
    pub comb_params: CombParams,
    pub comb_level: f32,
    pub fb: Feedback,
    pub fb_level: f32,
    pub comp: Compressor,
    pub comp_orbit: usize,
    pub sr: f32,
    silent_samples: u32,
    silence_holdoff: u32,
}

impl Orbit {
    pub fn new(sr: f32) -> Self {
        let silence_holdoff = (sr * SILENCE_HOLDOFF_SECS) as u32;
        Self {
            bus: [0.0; CHANNELS],
            delay: Delay::new(sr),
            delay_level: 0.0,
            dattorro: std::array::from_fn(|_| DattorroVerb::new(sr)),
            vital: VitalVerb::new(sr),
            reverb_params: ReverbParams::default(),
            verb_level: 0.0,
            comb: [Comb::default(); CHANNELS],
            comb_params: CombParams::default(),
            comb_level: 0.0,
            fb: Feedback::default(),
            fb_level: 0.0,
            comp: Compressor::default(),
            comp_orbit: 0,
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
        let has_input = self.bus[0] != 0.0 || self.bus[1] != 0.0;

        if has_input {
            self.silent_samples = 0;
        } else if self.silent_samples > self.silence_holdoff {
            return;
        }

        // Comb (per-channel mono resonator, shared params)
        if self.comb_level > 0.0 {
            let mut wet = [0.0_f32; CHANNELS];
            for (channel, w) in wet.iter_mut().enumerate() {
                *w = self.comb[channel].process(
                    self.bus[channel] * self.comb_level,
                    &self.comb_params,
                    self.sr,
                );
            }
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Feedback (stereo short delay with cross-channel, LFO + params on FX)
        if self.fb_level > 0.0 {
            let fb_in = [self.bus[0] * self.fb_level, self.bus[1] * self.fb_level];
            let wet = self.fb.process(fb_in, self.fb_level, self.sr);
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Delay (stereo)
        if self.delay_level > 0.0 {
            let delay_in = [
                self.bus[0] * self.delay_level,
                self.bus[1] * self.delay_level,
            ];
            let wet = self.delay.process(delay_in);
            self.bus[0] += wet[0];
            self.bus[1] += wet[1];
        }

        // Reverb — last in chain so it captures delay echoes
        if self.verb_level > 0.0 {
            let verb_in = [self.bus[0] * self.verb_level, self.bus[1] * self.verb_level];
            let rp = &self.reverb_params;
            let wet = match rp.verb_type {
                ReverbType::Plate => {
                    let mut out = [0.0; CHANNELS];
                    for (channel, vin) in verb_in.iter().enumerate() {
                        let w = self.dattorro[channel].process(*vin, rp);
                        out[0] += w[0];
                        out[1] += w[1];
                    }
                    out
                }
                ReverbType::Space => self.vital.process(verb_in, rp),
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
