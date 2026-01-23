//! Source generation - oscillators, samples, Plaits engines, spread mode.

use crate::fastmath::exp2f;
use crate::oscillator::Phasor;
use crate::plaits::PlaitsEngine;
use crate::sample::SampleInfo;
use crate::types::{freq2midi, Source, SubWave, BLOCK_SIZE, CHANNELS};
use mi_plaits_dsp::engine::{EngineParameters, TriggerState};

use super::Voice;

impl Voice {
    #[inline]
    pub(super) fn osc_at(&self, phase: f32, dt: f32) -> f32 {
        match self.params.sound {
            Source::Tri => Phasor::tri_at(phase, &self.params.shape),
            Source::Sine => Phasor::sine_at(phase, &self.params.shape),
            Source::Saw => Phasor::saw_at(phase, dt, &self.params.shape),
            Source::Zaw => Phasor::zaw_at(phase, &self.params.shape),
            Source::Pulse => Phasor::pulse_at(phase, dt, self.params.pw, &self.params.shape),
            Source::Pulze => Phasor::pulze_at(phase, self.params.pw, &self.params.shape),
            _ => 0.0,
        }
    }

    pub(super) fn run_source(
        &mut self,
        freq: f32,
        isr: f32,
        pool: &[f32],
        samples: &[SampleInfo],
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
    ) -> bool {
        match self.params.sound {
            Source::Sample => {
                if let Some(ref mut fs) = self.file_source {
                    if let Some(info) = samples.get(fs.sample_idx) {
                        if fs.is_done(info) {
                            return false;
                        }
                        for c in 0..CHANNELS {
                            self.ch[c] = fs.read(pool, info, c) * 0.2;
                        }
                        fs.advance(freq / 261.626);
                        return true;
                    }
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::WebSample => {
                if let Some(ref mut ws) = self.web_sample {
                    if ws.is_done() {
                        return false;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = ws.read(web_pcm, c) * 0.2;
                    }
                    ws.advance(freq / 261.626);
                    return true;
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::LiveInput => {
                let input_idx = sample_idx * CHANNELS;
                for c in 0..CHANNELS {
                    let idx = input_idx + c;
                    self.ch[c] = live_input.get(idx).copied().unwrap_or(0.0) * 0.2;
                }
            }
            Source::PlModal
            | Source::PlVa
            | Source::PlWs
            | Source::PlFm
            | Source::PlGrain
            | Source::PlAdd
            | Source::PlWt
            | Source::PlChord
            | Source::PlSwarm
            | Source::PlNoise
            | Source::PlBass
            | Source::PlSnare
            | Source::PlHat => {
                if self.plaits_idx >= BLOCK_SIZE {
                    let need_new = self
                        .plaits_engine
                        .as_ref()
                        .is_none_or(|e| e.source() != self.params.sound);
                    if need_new {
                        let sample_rate = 1.0 / isr;
                        self.plaits_engine = Some(PlaitsEngine::new(self.params.sound, sample_rate));
                    }
                    let engine = self.plaits_engine.as_mut().unwrap();

                    let trigger = if self.params.sound.is_plaits_percussion() {
                        TriggerState::Unpatched
                    } else {
                        let gate_high = self.params.gate > 0.5;
                        let t = if gate_high && !self.plaits_prev_gate {
                            TriggerState::RisingEdge
                        } else if gate_high {
                            TriggerState::High
                        } else {
                            TriggerState::Low
                        };
                        self.plaits_prev_gate = gate_high;
                        t
                    };

                    let params = EngineParameters {
                        trigger,
                        note: freq2midi(freq),
                        timbre: self.params.timbre,
                        morph: self.params.morph,
                        harmonics: self.params.harmonics,
                        accent: self.params.velocity,
                        a0_normalized: 55.0 * isr,
                    };

                    let mut already_enveloped = false;
                    engine.render(
                        &params,
                        &mut self.plaits_out,
                        &mut self.plaits_aux,
                        &mut already_enveloped,
                    );
                    self.plaits_idx = 0;
                }

                self.ch[0] = self.plaits_out[self.plaits_idx] * 0.2;
                self.ch[1] = self.ch[0];
                self.plaits_idx += 1;
            }
            _ => {
                let spread = self.params.spread;
                if spread > 0.0 {
                    self.run_spread(freq, isr);
                } else {
                    self.run_single_osc(freq, isr);
                }
                self.run_sub(freq, isr);
            }
        }
        true
    }

    fn run_spread(&mut self, freq: f32, isr: f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        const PAN: [f32; 3] = [0.3, 0.6, 0.9];

        // Center oscillator
        let dt_c = freq * isr;
        let phase_c = self.spread_phasors[3].phase;
        let center = self.osc_at(phase_c, dt_c);
        self.spread_phasors[3].phase = (phase_c + dt_c) % 1.0;
        left += center;
        right += center;

        // Symmetric pairs with parabolic detuning + stereo spread
        for i in 1..=3 {
            let detune_cents = (i * i) as f32 * self.params.spread;
            let ratio_up = exp2f(detune_cents / 1200.0);
            let ratio_down = exp2f(-detune_cents / 1200.0);

            let dt_up = freq * ratio_up * isr;
            let phase_up = self.spread_phasors[3 + i].phase;
            let voice_up = self.osc_at(phase_up, dt_up);
            self.spread_phasors[3 + i].phase = (phase_up + dt_up) % 1.0;

            let dt_down = freq * ratio_down * isr;
            let phase_down = self.spread_phasors[3 - i].phase;
            let voice_down = self.osc_at(phase_down, dt_down);
            self.spread_phasors[3 - i].phase = (phase_down + dt_down) % 1.0;

            let pan = PAN[i - 1];
            left += voice_down * (0.5 + pan * 0.5) + voice_up * (0.5 - pan * 0.5);
            right += voice_up * (0.5 + pan * 0.5) + voice_down * (0.5 - pan * 0.5);
        }

        // Store as mid/side - effects process mid, stereo restored later
        let mid = (left + right) / 2.0;
        let side = (left - right) / 2.0;
        self.ch[0] = mid / 4.0 * 0.2;
        self.spread_side = side / 4.0 * 0.2;
    }

    fn run_sub(&mut self, freq: f32, isr: f32) {
        if self.params.sub <= 0.0 {
            return;
        }
        let sub_freq = freq / (1 << self.params.sub_oct as u32) as f32;
        let sample = match self.params.sub_wave {
            SubWave::Sine => self.sub_phasor.sine(sub_freq, isr),
            SubWave::Tri => self.sub_phasor.tri(sub_freq, isr),
            SubWave::Square => self.sub_phasor.pulse(sub_freq, 0.5, isr),
        };
        self.ch[0] = (self.ch[0] + sample * self.params.sub * 0.2) / (1.0 + self.params.sub);
    }

    fn run_single_osc(&mut self, freq: f32, isr: f32) {
        self.ch[0] = match self.params.sound {
            Source::Tri => self.phasor.tri_shaped(freq, isr, &self.params.shape) * 0.2,
            Source::Sine => self.phasor.sine_shaped(freq, isr, &self.params.shape) * 0.2,
            Source::Saw => self.phasor.saw_shaped(freq, isr, &self.params.shape) * 0.2,
            Source::Zaw => self.phasor.zaw_shaped(freq, isr, &self.params.shape) * 0.2,
            Source::Pulse => {
                self.phasor
                    .pulse_shaped(freq, self.params.pw, isr, &self.params.shape)
                    * 0.2
            }
            Source::Pulze => {
                self.phasor
                    .pulze_shaped(freq, self.params.pw, isr, &self.params.shape)
                    * 0.2
            }
            Source::White => self.white() * 0.2,
            Source::Pink => {
                let w = self.white();
                self.pink_noise.next(w) * 0.2
            }
            Source::Brown => {
                let w = self.white();
                self.brown_noise.next(w) * 0.2
            }
            _ => 0.0,
        };
    }
}
