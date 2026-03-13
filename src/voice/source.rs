//! Source generation - oscillators, samples, spread mode.

use std::f32::consts::TAU;

use crate::dsp::{exp2f, sinf, Phasor};
#[cfg(not(feature = "native"))]
use crate::sampling::SampleInfo;
use crate::types::{Source, SubWave, CHANNELS};

use super::Voice;

const INV_MIDDLE_C: f32 = 1.0 / 261.626;

// log2(i) for i=1..32, precomputed to replace powf with a single exp2f
#[allow(clippy::approx_constant)]
const LOG2_TABLE: [f32; 32] = [
    0.0, 1.0, 1.584_963, 2.0, 2.321_928, 2.584_963, 2.807_355, 3.0,
    3.169_925, 3.321_928, 3.459_432, 3.584_963, 3.700_44, 3.807_355, 3.906_89, 4.0,
    4.087_463, 4.169_925, 4.247_928, 4.321_928, 4.392_317, 4.459_432, 4.523_562, 4.584_963,
    4.643_856, 4.700_44, 4.754_888, 4.807_355, 4.857_981, 4.906_89, 4.954_196, 5.0,
];

#[inline]
fn additive_at(phase: f32, dt: f32, timbre: f32, morph: f32, harmonics: f32, partials: f32) -> f32 {
    let tilt_exp = 3.0 * (1.0 - timbre);
    let stretch = harmonics * harmonics * 0.01;
    let max_n = partials.clamp(1.0, 32.0);
    let max_n_floor = max_n.floor() as u32;
    let max_n_ceil = max_n.ceil() as u32;
    let fract = max_n.fract();

    let phase_tau = phase * TAU;
    let gains = if morph < 0.5 {
        [morph * 2.0, 1.0] // [even, odd]
    } else {
        [1.0, (1.0 - morph) * 2.0]
    };

    let mut sum = 0.0_f32;
    let mut norm = 0.0_f32;

    for i in 1..=max_n_ceil {
        let fi = i as f32;
        let ratio = fi * (1.0 + stretch * (fi - 1.0));
        if dt * ratio >= 0.5 {
            break;
        }

        let mut amp = exp2f(-tilt_exp * LOG2_TABLE[i as usize - 1]);
        amp *= gains[(i & 1) as usize];

        if i > max_n_floor {
            amp *= fract;
        }

        sum += sinf(phase_tau * ratio) * amp;
        norm += amp;
    }

    if norm > 0.0 { sum / norm } else { 0.0 }
}

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
            Source::Add => {
                let shaped = self.params.shape.apply(phase);
                additive_at(shaped, dt, self.params.timbre, self.params.morph, self.params.harmonics, self.params.partials)
            }
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
        input_channels: usize,
    ) -> bool {
        match self.params.sound {
            Source::Gm => {
                if let Some(ref mut rs) = self.registry_sample {
                    let done = rs.is_done();
                    if done {
                        self.params.gate = 0.0;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = rs.read(c) * 0.7;
                    }
                    if !done {
                        rs.advance(freq / rs.root_freq);
                    }
                    self.nch = CHANNELS;
                    return true;
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::Sample => {
                let stretch = self.params.stretch;
                if stretch != 1.0 {
                    let pitch_ratio = (freq * INV_MIDDLE_C) as f64;
                    match (&self.registry_sample, &self.registry_sample_b) {
                        (Some(a), Some(b)) if self.sample_blend > 0.0 => {
                            if self.stretch.needs_init() {
                                self.stretch.reset(a.cursor_start(), a.cursor_end(), a.is_looping());
                            }
                            if self.stretch.is_done() { self.params.gate = 0.0; }
                            self.stretch.ensure_available(&a.data, stretch);
                            let blend = self.sample_blend;
                            for c in 0..CHANNELS {
                                let sa = self.stretch.read(c);
                                // Sample B reads from a fixed position (start of region)
                                let sb = b.data.read_interpolated(
                                    a.cursor_start() as f32, c,
                                );
                                self.ch[c] = (sa + blend * (sb - sa)) * 0.7;
                            }
                            self.stretch.advance(pitch_ratio);
                        }
                        (Some(rs), _) => {
                            if self.stretch.needs_init() {
                                self.stretch.reset(rs.cursor_start(), rs.cursor_end(), rs.is_looping());
                            }
                            if self.stretch.is_done() { self.params.gate = 0.0; }
                            self.stretch.ensure_available(&rs.data, stretch);
                            for c in 0..CHANNELS {
                                self.ch[c] = self.stretch.read(c) * 0.7;
                            }
                            self.stretch.advance(pitch_ratio);
                        }
                        _ => {
                            self.ch[0] = 0.0;
                            self.ch[1] = 0.0;
                        }
                    }
                    self.nch = CHANNELS;
                    return true;
                }
                let speed = freq * INV_MIDDLE_C;
                let blend = self.sample_blend;
                match (&mut self.registry_sample, &mut self.registry_sample_b) {
                    (Some(a), Some(b)) if blend > 0.0 => {
                        let done_a = a.is_done();
                        let done_b = b.is_done();
                        if done_a && done_b {
                            self.params.gate = 0.0;
                        }
                        for c in 0..CHANNELS {
                            self.ch[c] = (a.read(c) + blend * (b.read(c) - a.read(c))) * 0.7;
                        }
                        if !done_a { a.advance(speed); }
                        if !done_b { b.advance(speed); }
                        self.nch = CHANNELS;
                        return true;
                    }
                    (Some(rs), _) => {
                        let done = rs.is_done();
                        if done {
                            self.params.gate = 0.0;
                        }
                        for c in 0..CHANNELS {
                            self.ch[c] = rs.read(c) * 0.7;
                        }
                        if !done {
                            rs.advance(speed);
                        }
                        self.nch = CHANNELS;
                        return true;
                    }
                    _ => {
                        self.ch[0] = 0.0;
                        self.ch[1] = 0.0;
                    }
                }
            }
            Source::Wavetable => {
                self.nch = CHANNELS;
                self.run_wavetable(freq, isr);
            }
            Source::WebSample => {
                if let Some(ref mut ws) = self.web_sample {
                    let done = ws.is_done();
                    if done {
                        self.params.gate = 0.0;
                    }
                    for c in 0..CHANNELS {
                        self.ch[c] = ws.read(web_pcm, c) * 0.7;
                    }
                    if !done {
                        ws.advance(freq * INV_MIDDLE_C);
                    }
                    self.nch = CHANNELS;
                    return true;
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::LiveInput => {
                let nch = input_channels.max(1);
                if let Some(ch) = self.params.inchan {
                    self.nch = 1;
                    let idx = sample_idx * nch + ch.min(nch - 1);
                    self.ch[0] = live_input.get(idx).copied().unwrap_or(0.0) * 0.7;
                } else {
                    self.nch = CHANNELS;
                    let base = sample_idx * nch;
                    self.ch[0] = live_input.get(base).copied().unwrap_or(0.0) * 0.7;
                    self.ch[1] = live_input.get(base + 1.min(nch - 1)).copied().unwrap_or(0.0) * 0.7;
                }
            }
            Source::Kick | Source::Snare | Source::Hat | Source::Tom
            | Source::Rim | Source::Cowbell | Source::Cymbal => {
                self.nch = 1;
                self.run_drum(freq, isr);
            }
            _ => {
                self.nch = 1;
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
        input_channels: usize,
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
                            self.ch[c] = fs.read(pool, channels, info.offset, c) * 0.7;
                        }
                        if !done {
                            fs.advance(freq * INV_MIDDLE_C);
                        }
                        self.nch = CHANNELS;
                        return true;
                    }
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::Wavetable => {
                self.nch = CHANNELS;
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
                        self.ch[c] = ws.read(web_pcm, c) * 0.7;
                    }
                    if !done {
                        ws.advance(freq * INV_MIDDLE_C);
                    }
                    self.nch = CHANNELS;
                    return true;
                }
                self.ch[0] = 0.0;
                self.ch[1] = 0.0;
            }
            Source::LiveInput => {
                let nch = input_channels.max(1);
                if let Some(ch) = self.params.inchan {
                    self.nch = 1;
                    let idx = sample_idx * nch + ch.min(nch - 1);
                    self.ch[0] = live_input.get(idx).copied().unwrap_or(0.0) * 0.7;
                } else {
                    self.nch = CHANNELS;
                    let base = sample_idx * nch;
                    self.ch[0] = live_input.get(base).copied().unwrap_or(0.0) * 0.7;
                    self.ch[1] = live_input.get(base + 1.min(nch - 1)).copied().unwrap_or(0.0) * 0.7;
                }
            }
            Source::Kick | Source::Snare | Source::Hat | Source::Tom
            | Source::Rim | Source::Cowbell | Source::Cymbal => {
                self.nch = 1;
                self.run_drum(freq, isr);
            }
            _ => {
                self.nch = 1;
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
        self.ch[0] = mid / 4.0 * 0.5;
        self.spread_side = side / 4.0 * 0.5;
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
        self.ch[0] = (self.ch[0] + sample * self.params.sub * 0.5) / (1.0 + self.params.sub);
    }

    fn run_single_osc(&mut self, freq: f32, isr: f32) {
        self.ch[0] = match self.params.sound {
            Source::Tri => self.phasor.tri_shaped(freq, isr, &self.params.shape) * 0.5,
            Source::Sine => self.phasor.sine_shaped(freq, isr, &self.params.shape) * 0.5,
            Source::Saw => self.phasor.saw_shaped(freq, isr, &self.params.shape) * 0.5,
            Source::Zaw => self.phasor.zaw_shaped(freq, isr, &self.params.shape) * 0.5,
            Source::Pulse => {
                self.phasor
                    .pulse_shaped(freq, self.params.pw, isr, &self.params.shape)
                    * 0.5
            }
            Source::Pulze => {
                self.phasor
                    .pulze_shaped(freq, self.params.pw, isr, &self.params.shape)
                    * 0.5
            }
            Source::Add => {
                let dt = freq * isr;
                let phase = if self.params.shape.is_active() {
                    self.params.shape.apply(self.phasor.phase)
                } else {
                    self.phasor.phase
                };
                let s = additive_at(phase, dt, self.params.timbre, self.params.morph, self.params.harmonics, self.params.partials);
                self.phasor.update(freq, isr);
                s * 0.5
            }
            Source::White => self.white() * 0.5,
            Source::Pink => {
                let w = self.white();
                self.pink_noise.next(w) * 0.5
            }
            Source::Brown => {
                let w = self.white();
                self.brown_noise.next(w) * 0.5
            }
            _ => 0.0,
        };
    }

    #[cfg(feature = "native")]
    fn run_wavetable(&mut self, freq: f32, isr: f32) {
        // Compute modulated scan before borrowing registry_sample
        let scan = self.get_modulated_scan();

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
                self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
            }

            self.phasor.update(freq, isr);
        } else {
            self.ch[0] = 0.0;
            self.ch[1] = 0.0;
        }
    }

    fn get_modulated_scan(&self) -> f32 {
        self.params.scan.clamp(0.0, 1.0)
    }

    #[cfg(not(feature = "native"))]
    fn run_wavetable_wasm(&mut self, freq: f32, isr: f32, pool: &[f32], samples: &[SampleInfo]) {
        let scan = self.get_modulated_scan();

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
                    self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
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
        let scan = self.get_modulated_scan();

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
                self.ch[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
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
