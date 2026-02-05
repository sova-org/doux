//! Source generation - oscillators, samples, Plaits engines, spread mode.

use crate::dsp::{exp2f, Phasor};
use crate::plaits::PlaitsEngine;
#[cfg(not(feature = "native"))]
use crate::sampling::SampleInfo;
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

    #[cfg(feature = "native")]
    pub(super) fn run_source(
        &mut self,
        freq: f32,
        isr: f32,
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
    ) -> bool {
        match self.params.sound {
            Source::Sample => {
                if let Some(ref mut rs) = self.registry_sample {
                    let done = rs.is_done();
                    if done {
                        self.params.gate = 0.0;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = rs.read(c) * 0.2;
                    }
                    if !done {
                        rs.advance(freq / 261.626);
                    }
                    return true;
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::Wavetable => {
                self.run_wavetable(freq, isr);
            }
            Source::WebSample => {
                if let Some(ref mut ws) = self.web_sample {
                    let done = ws.is_done();
                    if done {
                        self.params.gate = 0.0;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = ws.read(web_pcm, c) * 0.2;
                    }
                    if !done {
                        ws.advance(freq / 261.626);
                    }
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
                self.run_plaits(freq, isr);
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

    #[cfg(not(feature = "native"))]
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
                        let done = fs.is_done();
                        if done {
                            self.params.gate = 0.0;
                        }
                        let channels = info.channels as usize;
                        for c in 0..CHANNELS {
                            self.ch[c] = fs.read(pool, channels, info.offset, c) * 0.2;
                        }
                        if !done {
                            fs.advance(freq / 261.626);
                        }
                        return true;
                    }
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::Wavetable => {
                if self.web_sample.is_some() {
                    self.run_wavetable_web(freq, isr, web_pcm);
                } else {
                    self.run_wavetable_wasm(freq, isr, pool, samples);
                }
            }
            Source::WebSample => {
                if let Some(ref mut ws) = self.web_sample {
                    let done = ws.is_done();
                    if done {
                        self.params.gate = 0.0;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = ws.read(web_pcm, c) * 0.2;
                    }
                    if !done {
                        ws.advance(freq / 261.626);
                    }
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
                self.run_plaits(freq, isr);
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

    fn run_plaits(&mut self, freq: f32, isr: f32) {
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

    fn run_spread(&mut self, freq: f32, isr: f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        const PAN: [f32; 3] = [0.3, 0.6, 0.9];

        let dt_c = freq * isr;
        let phase_c = self.spread_phasors[3].phase;
        let center = self.osc_at(phase_c, dt_c);
        self.spread_phasors[3].phase = (phase_c + dt_c).fract();
        left += center;
        right += center;

        for i in 1..=3 {
            let detune_cents = (i * i) as f32 * self.params.spread;
            let ratio_up = exp2f(detune_cents / 1200.0);
            let ratio_down = 1.0 / ratio_up;

            let dt_up = freq * ratio_up * isr;
            let phase_up = self.spread_phasors[3 + i].phase;
            let voice_up = self.osc_at(phase_up, dt_up);
            self.spread_phasors[3 + i].phase = (phase_up + dt_up).fract();

            let dt_down = freq * ratio_down * isr;
            let phase_down = self.spread_phasors[3 - i].phase;
            let voice_down = self.osc_at(phase_down, dt_down);
            self.spread_phasors[3 - i].phase = (phase_down + dt_down).fract();

            let pan = PAN[i - 1];
            left += voice_down * (0.5 + pan * 0.5) + voice_up * (0.5 - pan * 0.5);
            right += voice_up * (0.5 + pan * 0.5) + voice_down * (0.5 - pan * 0.5);
        }

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

    #[cfg(feature = "native")]
    fn run_wavetable(&mut self, freq: f32, isr: f32) {
        // Compute modulated scan before borrowing registry_sample
        let scan = self.get_modulated_scan(isr);

        if let Some(ref rs) = self.registry_sample {
            let frame_count = rs.data.frame_count as f32;

            let cycle_len = if self.params.wt_cycle_len > 0 {
                self.params.wt_cycle_len as f32
            } else {
                frame_count
            };

            let num_cycles = (frame_count / cycle_len).floor().max(1.0);

            let phase = if self.params.shape.is_active() {
                self.params.shape.apply(self.phasor.phase)
            } else {
                self.phasor.phase
            };

            let scan_pos = scan * (num_cycles - 1.0);
            let cycle_a = scan_pos.floor() as usize;
            let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
            let blend = scan_pos.fract();

            let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
            let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

            let channels = rs.data.channels as usize;
            for c in 0..CHANNELS {
                let ch = c.min(channels - 1);
                let sample_a = rs.data.read_interpolated(pos_a, ch);
                let sample_b = rs.data.read_interpolated(pos_b, ch);
                self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.2;
            }

            self.phasor.update(freq, isr);
        } else {
            self.ch[0] = 0.0;
            self.ch[1] = 0.0;
        }
    }

    fn get_modulated_scan(&mut self, isr: f32) -> f32 {
        let mut scan = self.params.scan;

        if self.params.scanlfo > 0.0 {
            let lfo = self
                .scan_lfo
                .lfo(self.params.scanshape, self.params.scanlfo, isr);
            scan += lfo * self.params.scandepth * 0.5;
        }

        scan.clamp(0.0, 1.0)
    }

    #[cfg(not(feature = "native"))]
    fn run_wavetable_wasm(&mut self, freq: f32, isr: f32, pool: &[f32], samples: &[SampleInfo]) {
        let scan = self.get_modulated_scan(isr);

        if let Some(ref fs) = self.file_source {
            if let Some(info) = samples.get(fs.sample_idx) {
                let frame_count = info.frames as f32;
                let channels = info.channels as usize;
                let offset = info.offset;

                let cycle_len = if self.params.wt_cycle_len > 0 {
                    self.params.wt_cycle_len as f32
                } else {
                    frame_count
                };

                let num_cycles = (frame_count / cycle_len).floor().max(1.0);

                let phase = if self.params.shape.is_active() {
                    self.params.shape.apply(self.phasor.phase)
                } else {
                    self.phasor.phase
                };

                let scan_pos = scan * (num_cycles - 1.0);
                let cycle_a = scan_pos.floor() as usize;
                let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
                let blend = scan_pos.fract();

                let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
                let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

                let frames = frame_count as usize;
                for c in 0..CHANNELS {
                    let ch = c.min(channels - 1);
                    let sample_a = read_interpolated(pool, offset, channels, frames, pos_a, ch);
                    let sample_b = read_interpolated(pool, offset, channels, frames, pos_b, ch);
                    self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.2;
                }

                self.phasor.update(freq, isr);
                return;
            }
        }
        self.ch[0] = 0.0;
        self.ch[1] = 0.0;
    }

    #[cfg(not(feature = "native"))]
    fn run_wavetable_web(&mut self, freq: f32, isr: f32, web_pcm: &[f32]) {
        let scan = self.get_modulated_scan(isr);

        if let Some(ref ws) = self.web_sample {
            let frame_count = ws.frame_count();
            let channels = ws.info.channels as usize;
            let offset = ws.info.offset;

            let cycle_len = if self.params.wt_cycle_len > 0 {
                self.params.wt_cycle_len as f32
            } else {
                frame_count
            };

            let num_cycles = (frame_count / cycle_len).floor().max(1.0);

            let phase = if self.params.shape.is_active() {
                self.params.shape.apply(self.phasor.phase)
            } else {
                self.phasor.phase
            };

            let scan_pos = scan * (num_cycles - 1.0);
            let cycle_a = scan_pos.floor() as usize;
            let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
            let blend = scan_pos.fract();

            let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
            let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

            let frames = frame_count as usize;
            for c in 0..CHANNELS {
                let ch = c.min(channels - 1);
                let sample_a = read_interpolated(web_pcm, offset, channels, frames, pos_a, ch);
                let sample_b = read_interpolated(web_pcm, offset, channels, frames, pos_b, ch);
                self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.2;
            }

            self.phasor.update(freq, isr);
            return;
        }
        self.ch[0] = 0.0;
        self.ch[1] = 0.0;
    }
}

#[cfg(not(feature = "native"))]
#[inline]
fn read_interpolated(
    pool: &[f32],
    offset: usize,
    channels: usize,
    frames: usize,
    pos: f32,
    channel: usize,
) -> f32 {
    let idx0 = pos.floor() as usize;
    let idx1 = (idx0 + 1) % frames;
    let frac = pos.fract();

    let i0 = offset + idx0 * channels + channel;
    let i1 = offset + idx1 * channels + channel;

    let s0 = pool.get(i0).copied().unwrap_or(0.0);
    let s1 = pool.get(i1).copied().unwrap_or(0.0);
    s0 + frac * (s1 - s0)
}
