//! Source generation - oscillators, samples, spread mode.

use arf::graph::{GATE_LANE, NOTEFREQ_LANE, VEL_LANE};

use crate::dsp::oscillator::{blamp_post_kink, blamp_pre_kink, blep_post_step, blep_pre_step};
#[cfg(feature = "native")]
use crate::dsp::{exp2f, log2f};
use crate::dsp::{PhaseShape, Phasor};
#[cfg(not(feature = "native"))]
use crate::sampling::SampleInfo;
use crate::types::{Source, SubWave, SyncMode, CHANNELS};

use super::Voice;

const INV_MIDDLE_C: f32 = 1.0 / 261.626;
const SYNC_RATIO_EPS: f32 = 1e-4;

#[inline]
fn wrap_phase(phase: f32) -> f32 {
    if phase >= 1.0 {
        phase - 1.0
    } else if phase < 0.0 {
        phase + 1.0
    } else {
        phase
    }
}

/// Wraps any finite phase value into `[0, 1)`. Use when the offset may be
/// outside `[-1, 1)` (e.g. large FM phase-mod depth).
#[inline]
fn wrap_phase_any(phase: f32) -> f32 {
    phase - phase.floor()
}

/// Smoothstep crossfade for wavetable scan blending. Constant-sum for
/// correlated adjacent cycles, with zero derivative at cycle boundaries so
/// scan modulation has no corner when crossing frames.
#[inline]
fn scan_xfade(blend: f32) -> f32 {
    blend * blend * (3.0 - 2.0 * blend)
}

#[inline]
fn osc_morph_at(phase: f32, dt: f32, wave: f32, shape: &PhaseShape) -> f32 {
    let w = wave.clamp(0.0, 1.0) * 3.0;
    let segment = (w as u32).min(2);
    let t = w - segment as f32;
    match segment {
        0 => {
            let a = Phasor::sine_at(phase, shape);
            let b = Phasor::tri_at(phase, dt, shape);
            a + t * (b - a)
        }
        1 => {
            let a = Phasor::tri_at(phase, dt, shape);
            let b = Phasor::saw_at(phase, dt, shape);
            a + t * (b - a)
        }
        _ => {
            let a = Phasor::saw_at(phase, dt, shape);
            let b = Phasor::pulse_at(phase, dt, 0.5, shape);
            a + t * (b - a)
        }
    }
}

impl Voice {
    #[inline]
    fn shape_phase(&self, phase: f32) -> f32 {
        if self.shape_active {
            self.params.shape.apply(phase)
        } else {
            phase
        }
    }

    #[inline]
    pub(super) fn osc_at(&self, phase: f32, dt: f32) -> f32 {
        match self.params.sound {
            Source::Tri => Phasor::tri_at(phase, dt, &self.params.shape),
            Source::Sine => Phasor::sine_at(phase, &self.params.shape),
            Source::Saw => Phasor::saw_at(phase, dt, &self.params.shape),
            Source::Zaw => Phasor::zaw_at(phase, &self.params.shape),
            Source::Pulse => Phasor::pulse_at(phase, dt, self.params.pw, &self.params.shape),
            Source::Pulze => Phasor::pulze_at(phase, self.params.pw, &self.params.shape),
            Source::Osc => osc_morph_at(phase, dt, self.params.wave, &self.params.shape),
            _ => 0.0,
        }
    }

    /// Per-sample voice DSP for a block. Dispatch on `self.params.sound` hoists
    /// outside the per-sample loop. Each iteration:
    /// `apply_mods → fm tick → vib → source body → filters+effects`.
    /// Mirrors `main`'s `Voice::process` ordering exactly.
    #[cfg(feature = "native")]
    #[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
    pub(crate) fn run_source_block(
        &mut self,
        env: &[f32],
        isr: f32,
        n: usize,
        _web_pcm: &[f32],
        _sample_idx: usize,
        live_input: &[f32],
        input_channels: usize,
    ) -> usize {
        self.build_stage_program();
        match self.params.sound {
            Source::Gm => {
                self.nch = CHANNELS;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let releasing = self.dahdsr.is_releasing();
                    if let Some(ref mut rs) = self.registry_sample {
                        // SF2 sample mode 3: once the voice enters release, stop
                        // looping and let the cursor run out the post-loop tail.
                        if rs.loop_until_release && releasing && rs.is_looping() {
                            rs.set_loop(0.0, 0.0);
                        }
                        let done = rs.is_done();
                        if done {
                            self.dahdsr.force_release();
                        }
                        // Mono samples read ch0 for both; stereo (linked L/R) pairs
                        // read their two interleaved channels.
                        let gain = rs.attenuation * 0.7;
                        self.scratch[i][0] = rs.read(0) * gain;
                        self.scratch[i][1] = rs.read(1) * gain;
                        if !done {
                            let ratio = freq / rs.root_freq;
                            let pitch = if rs.scale_tuning == 1.0 {
                                ratio
                            } else {
                                exp2f(rs.scale_tuning * log2f(ratio))
                            };
                            // Fold native→device rate conversion into the speed so
                            // the sample plays at its native rate without an
                            // up-front resample.
                            rs.advance(pitch * rs.sr_ratio);
                        }
                    } else {
                        self.scratch[i] = [0.0; CHANNELS];
                    }
                }
            }
            Source::Sample => {
                self.nch = CHANNELS;
                let stretch = self.params.stretch;
                if stretch != 1.0 {
                    for i in 0..n {
                        let freq = self.tick_pre(isr, i);
                        let pitch_ratio = (freq * INV_MIDDLE_C) as f64;
                        let blend = self.sample_blend;
                        match (&self.registry_sample, &self.registry_sample_b) {
                            (Some(_), Some(_)) if blend > 0.0 => {
                                if self.stretch.needs_init() {
                                    let a = self.registry_sample.as_ref().unwrap();
                                    self.stretch.reset(
                                        a.cursor_start(),
                                        a.cursor_end(),
                                        a.is_looping(),
                                    );
                                }
                                if self.stretch.is_done() {
                                    self.dahdsr.force_release();
                                }
                                let a = self.registry_sample.as_ref().unwrap();
                                self.stretch.ensure_available(&a.data, stretch);
                                let a_start = a.cursor_start() as f32;
                                let sa0 = self.stretch.read(0);
                                let sa1 = self.stretch.read(1);
                                let b = self.registry_sample_b.as_ref().unwrap();
                                let sb0 = b.data.read_interpolated(a_start, 0);
                                let sb1 = b.data.read_interpolated(a_start, 1);
                                self.scratch[i][0] = (sa0 + blend * (sb0 - sa0)) * 0.7;
                                self.scratch[i][1] = (sa1 + blend * (sb1 - sa1)) * 0.7;
                                self.stretch.advance(pitch_ratio);
                            }
                            (Some(_), _) => {
                                if self.stretch.needs_init() {
                                    let rs = self.registry_sample.as_ref().unwrap();
                                    self.stretch.reset(
                                        rs.cursor_start(),
                                        rs.cursor_end(),
                                        rs.is_looping(),
                                    );
                                }
                                if self.stretch.is_done() {
                                    self.dahdsr.force_release();
                                }
                                let rs = self.registry_sample.as_ref().unwrap();
                                self.stretch.ensure_available(&rs.data, stretch);
                                self.scratch[i][0] = self.stretch.read(0) * 0.7;
                                self.scratch[i][1] = self.stretch.read(1) * 0.7;
                                self.stretch.advance(pitch_ratio);
                            }
                            _ => {
                                self.scratch[i] = [0.0; CHANNELS];
                            }
                        }
                    }
                    self.finish_block(env, n, isr);
                    return n;
                }
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let speed = freq * INV_MIDDLE_C;
                    let blend = self.sample_blend;
                    match (&mut self.registry_sample, &mut self.registry_sample_b) {
                        (Some(a), Some(b)) if blend > 0.0 => {
                            let done_a = a.is_done();
                            let done_b = b.is_done();
                            if done_a && done_b {
                                self.dahdsr.force_release();
                            }
                            self.scratch[i][0] =
                                (a.read(0) + blend * (b.read(0) - a.read(0))) * 0.7;
                            self.scratch[i][1] =
                                (a.read(1) + blend * (b.read(1) - a.read(1))) * 0.7;
                            if !done_a {
                                a.advance(speed);
                            }
                            if !done_b {
                                b.advance(speed);
                            }
                        }
                        (Some(rs), _) => {
                            let done = rs.is_done();
                            if done {
                                self.dahdsr.force_release();
                            }
                            self.scratch[i][0] = rs.read(0) * 0.7;
                            self.scratch[i][1] = rs.read(1) * 0.7;
                            if !done {
                                rs.advance(speed);
                            }
                        }
                        _ => {
                            self.scratch[i] = [0.0; CHANNELS];
                        }
                    }
                }
            }
            Source::Wavetable => {
                self.nch = CHANNELS;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    self.scratch[i] = self.run_wavetable(freq, isr);
                }
            }
            Source::WebSample => {
                self.nch = CHANNELS;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    if let Some(ref mut ws) = self.web_sample {
                        let done = ws.is_done();
                        if done {
                            self.dahdsr.force_release();
                        }
                        self.scratch[i][0] = ws.read(_web_pcm, 0) * 0.7;
                        self.scratch[i][1] = ws.read(_web_pcm, 1) * 0.7;
                        if !done {
                            ws.advance(freq * INV_MIDDLE_C);
                        }
                    } else {
                        self.scratch[i] = [0.0; CHANNELS];
                    }
                }
            }
            Source::LiveInput => {
                let input_nch = input_channels.max(1);
                if let Some(chan) = self.params.inchan {
                    self.nch = 1;
                    let ch = chan.min(input_nch - 1);
                    for i in 0..n {
                        let _ = self.tick_pre(isr, i);
                        let idx = (_sample_idx + i) * input_nch + ch;
                        let v = live_input.get(idx).copied().unwrap_or(0.0) * 0.7;
                        self.scratch[i][0] = v;
                        self.scratch[i][1] = 0.0;
                    }
                } else {
                    self.nch = CHANNELS;
                    let right_off = 1.min(input_nch - 1);
                    for i in 0..n {
                        let _ = self.tick_pre(isr, i);
                        let base = (_sample_idx + i) * input_nch;
                        self.scratch[i][0] = live_input.get(base).copied().unwrap_or(0.0) * 0.7;
                        self.scratch[i][1] =
                            live_input.get(base + right_off).copied().unwrap_or(0.0) * 0.7;
                    }
                }
            }
            Source::Kick
            | Source::Snare
            | Source::Hat
            | Source::Tom
            | Source::Rim
            | Source::Cowbell
            | Source::Cymbal
            | Source::Clap => {
                self.nch = 2;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let s = self.run_drum(freq, isr);
                    self.scratch[i][0] = s[0];
                    self.scratch[i][1] = s[1];
                    self.time += isr;
                }
            }
            Source::Pluck => {
                self.nch = 1;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let s_main = self.run_pluck(freq, isr);
                    let s = self.run_sub(freq, isr, s_main);
                    self.scratch[i][0] = s;
                    self.scratch[i][1] = 0.0;
                }
            }
            Source::Arf => self.run_arf_block(n, isr),
            _ => {
                self.nch = 1;
                let spread = self.params.spread;
                if spread > 0.0 {
                    for i in 0..n {
                        let freq = self.tick_pre(isr, i);
                        let (mid, side) = self.run_spread(freq, isr);
                        let s = self.run_sub(freq, isr, mid);
                        self.scratch[i][0] = s;
                        self.scratch[i][1] = side;
                    }
                } else {
                    for i in 0..n {
                        let freq = self.tick_pre(isr, i);
                        let s_main = self.run_single_osc(freq, isr);
                        let s = self.run_sub(freq, isr, s_main);
                        self.scratch[i][0] = s;
                        self.scratch[i][1] = 0.0;
                    }
                }
            }
        }
        self.finish_block(env, n, isr);
        n
    }

    /// The `Source::Arf` body, shared by the native and wasm dispatchers.
    ///
    /// Per sample: the `tick_pre` freq (vibrato, ModChain-on-freq, glide)
    /// lands in the notefreq lane and the doux envelope's release state gates
    /// the gate lane, then the Vm ticks one frame — the VM re-reads the
    /// control plane every frame, so arf graphs get the same per-sample
    /// modulation as native sources. Velocity is a note property, latched
    /// once per block. No patch handle (registry miss raced an install, or a
    /// bare `s/arf`) renders silence.
    fn run_arf_block(&mut self, n: usize, isr: f32) {
        self.nch = self
            .patch
            .as_ref()
            .map_or(1, |vp| vp.entry.program().audio_channels());
        if let Some(ref mut vp) = self.patch {
            vp.control[VEL_LANE] = self.params.velocity;
        }
        let mut bad = false;
        for i in 0..n {
            let freq = self.tick_pre(isr, i);
            let gate = if self.dahdsr.is_releasing() { 0.0 } else { 1.0 };
            if let Some(ref mut vp) = self.patch {
                vp.control[NOTEFREQ_LANE] = freq;
                vp.control[GATE_LANE] = gate;
                let program = vp.entry.program();
                let width = program.audio_channels().min(CHANNELS);
                let mut frame = [0.0f32; CHANNELS];
                vp.vm.tick_frame(
                    program,
                    vp.frame_pos,
                    &[],
                    &vp.control[..program.control_len()],
                    &mut frame[..width],
                );
                vp.frame_pos += 1;
                // One unscrubbed non-finite sample would permanently poison
                // the master DC-blocker downstream. 0.7 is the same headroom
                // scale as the sample sources.
                bad |= crate::patch::scrub_non_finite(&mut frame);
                self.scratch[i][0] = frame[0] * 0.7;
                self.scratch[i][1] = frame[1] * 0.7;
            } else {
                self.scratch[i] = [0.0; CHANNELS];
            }
        }
        // Flag a latched source Vm for the heal path (`gen_block`). A held or
        // same-patch-retriggered voice keeps its Vm, so it never self-heals
        // without this.
        self.patch_poisoned = bad;
    }

    #[cfg(not(feature = "native"))]
    #[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
    pub(crate) fn run_source_block(
        &mut self,
        env: &[f32],
        isr: f32,
        n: usize,
        pool: &[f32],
        samples: &[SampleInfo],
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
        input_channels: usize,
    ) -> usize {
        self.build_stage_program();
        match self.params.sound {
            Source::Sample => {
                self.nch = CHANNELS;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let mut wrote = false;
                    if let Some(ref mut fs) = self.file_source {
                        if let Some(info) = samples.get(fs.sample_idx) {
                            let done = fs.is_done();
                            if done {
                                self.dahdsr.force_release();
                            }
                            let channels = info.channels as usize;
                            self.scratch[i][0] = fs.read(pool, channels, info.offset, 0) * 0.7;
                            self.scratch[i][1] = fs.read(pool, channels, info.offset, 1) * 0.7;
                            if !done {
                                fs.advance(freq * INV_MIDDLE_C);
                            }
                            wrote = true;
                        }
                    }
                    if !wrote {
                        self.scratch[i] = [0.0; CHANNELS];
                    }
                }
            }
            Source::Wavetable => {
                self.nch = CHANNELS;
                let use_web = self.web_sample.is_some();
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let frame = if use_web {
                        self.run_wavetable_web(freq, isr, web_pcm)
                    } else {
                        self.run_wavetable_wasm(freq, isr, pool, samples)
                    };
                    self.scratch[i] = frame;
                }
            }
            Source::WebSample => {
                self.nch = CHANNELS;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    if let Some(ref mut ws) = self.web_sample {
                        let done = ws.is_done();
                        if done {
                            self.dahdsr.force_release();
                        }
                        self.scratch[i][0] = ws.read(web_pcm, 0) * 0.7;
                        self.scratch[i][1] = ws.read(web_pcm, 1) * 0.7;
                        if !done {
                            ws.advance(freq * INV_MIDDLE_C);
                        }
                    } else {
                        self.scratch[i] = [0.0; CHANNELS];
                    }
                }
            }
            Source::LiveInput => {
                let input_nch = input_channels.max(1);
                if let Some(chan) = self.params.inchan {
                    self.nch = 1;
                    let ch = chan.min(input_nch - 1);
                    for i in 0..n {
                        let _ = self.tick_pre(isr, i);
                        let idx = (sample_idx + i) * input_nch + ch;
                        let v = live_input.get(idx).copied().unwrap_or(0.0) * 0.7;
                        self.scratch[i][0] = v;
                        self.scratch[i][1] = 0.0;
                    }
                } else {
                    self.nch = CHANNELS;
                    let right_off = 1.min(input_nch - 1);
                    for i in 0..n {
                        let _ = self.tick_pre(isr, i);
                        let base = (sample_idx + i) * input_nch;
                        self.scratch[i][0] = live_input.get(base).copied().unwrap_or(0.0) * 0.7;
                        self.scratch[i][1] =
                            live_input.get(base + right_off).copied().unwrap_or(0.0) * 0.7;
                    }
                }
            }
            Source::Kick
            | Source::Snare
            | Source::Hat
            | Source::Tom
            | Source::Rim
            | Source::Cowbell
            | Source::Cymbal
            | Source::Clap => {
                self.nch = 2;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let s = self.run_drum(freq, isr);
                    self.scratch[i][0] = s[0];
                    self.scratch[i][1] = s[1];
                    self.time += isr;
                }
            }
            Source::Pluck => {
                self.nch = 1;
                for i in 0..n {
                    let freq = self.tick_pre(isr, i);
                    let s_main = self.run_pluck(freq, isr);
                    let s = self.run_sub(freq, isr, s_main);
                    self.scratch[i][0] = s;
                    self.scratch[i][1] = 0.0;
                }
            }
            Source::Arf => self.run_arf_block(n, isr),
            _ => {
                self.nch = 1;
                let spread = self.params.spread;
                if spread > 0.0 {
                    for i in 0..n {
                        let freq = self.tick_pre(isr, i);
                        let (mid, side) = self.run_spread(freq, isr);
                        let s = self.run_sub(freq, isr, mid);
                        self.scratch[i][0] = s;
                        self.scratch[i][1] = side;
                    }
                } else {
                    for i in 0..n {
                        let freq = self.tick_pre(isr, i);
                        let s_main = self.run_single_osc(freq, isr);
                        let s = self.run_sub(freq, isr, s_main);
                        self.scratch[i][0] = s;
                        self.scratch[i][1] = 0.0;
                    }
                }
            }
        }
        self.finish_block(env, n, isr);
        n
    }

    fn run_spread(&mut self, freq: f32, isr: f32) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        const PAN: [f32; 3] = [0.3, 0.6, 0.9];
        let ratios = *self.spread_detune_ratios();

        let pm = self.fm_phase_mod;
        let dt_c = freq * isr;
        let phase_c = self.spread_phasors[3].phase;
        let center = self.osc_at(wrap_phase_any(phase_c + pm), dt_c);
        self.spread_phasors[3].phase = wrap_phase(phase_c + dt_c);
        left += center;
        right += center;

        for i in 1..=3 {
            let ratio_up = ratios[i - 1];
            let ratio_down = 1.0 / ratio_up;

            let dt_up = freq * ratio_up * isr;
            let phase_up = self.spread_phasors[3 + i].phase;
            let voice_up = self.osc_at(wrap_phase_any(phase_up + pm), dt_up);
            self.spread_phasors[3 + i].phase = wrap_phase(phase_up + dt_up);

            let dt_down = freq * ratio_down * isr;
            let phase_down = self.spread_phasors[3 - i].phase;
            let voice_down = self.osc_at(wrap_phase_any(phase_down + pm), dt_down);
            self.spread_phasors[3 - i].phase = wrap_phase(phase_down + dt_down);

            let pan = PAN[i - 1];
            left += voice_down * (0.5 + pan * 0.5) + voice_up * (0.5 - pan * 0.5);
            right += voice_up * (0.5 + pan * 0.5) + voice_down * (0.5 - pan * 0.5);
        }

        let mid = (left + right) / 2.0;
        let side = (left - right) / 2.0;
        (mid / 4.0 * 0.5, side / 4.0 * 0.5)
    }

    fn run_sub(&mut self, freq: f32, isr: f32, current: f32) -> f32 {
        if self.params.sub <= 0.0 {
            return current;
        }
        let sub_freq = freq / (1 << self.params.sub_oct as u32) as f32;
        let sample = match self.params.sub_wave {
            SubWave::Sine => self.sub_phasor.sine(sub_freq, isr),
            SubWave::Tri => self.sub_phasor.tri(sub_freq, isr),
            SubWave::Square => self.sub_phasor.pulse(sub_freq, 0.5, isr),
        };
        (current + sample * self.params.sub * 0.5) / (1.0 + self.params.sub)
    }

    fn run_single_osc(&mut self, freq: f32, isr: f32) -> f32 {
        let ratio = self.params.sync_ratio;
        if ratio <= 1.0 + SYNC_RATIO_EPS {
            return self.generate_main_osc(freq, isr);
        }

        let master_dt = freq * isr;
        let slave_dt = master_dt * ratio;
        let prev = self.sync_phasor.phase;
        self.sync_phasor.update(freq, isr);
        let master_wrapped = self.sync_phasor.phase < prev;
        let wrap_frac = if master_wrapped && master_dt > 0.0 {
            self.sync_phasor.phase / master_dt
        } else {
            0.0
        };

        let aa_saw = matches!(self.params.sound, Source::Saw);
        let next_wrap_frac = if aa_saw && master_dt > 0.0 {
            let overshoot = self.sync_phasor.phase + master_dt - 1.0;
            if overshoot >= 0.0 {
                Some(overshoot / master_dt)
            } else {
                None
            }
        } else {
            None
        };

        match self.params.sync_mode {
            SyncMode::Hard => {
                let phase_before = self.phasor.phase;
                let p = wrap_phase(self.params.sync_phase + slave_dt * wrap_frac);
                if master_wrapped {
                    self.phasor.phase = p;
                }
                let mut sample = self.generate_main_osc(freq * ratio, isr);

                if master_wrapped && aa_saw {
                    let phase_at_wrap = wrap_phase(phase_before + (1.0 - wrap_frac) * slave_dt);
                    let h = 2.0 * (p - phase_at_wrap);
                    // saw_shaped's natural-wrap polyBLEP fires on the post-reset
                    // phase assuming a −2 step; cancel it before applying the
                    // correct lobe for the actual step height.
                    let d = 1.0 - wrap_frac;
                    let natural = if p < slave_dt { 0.5 * d * d } else { 0.0 };
                    sample += 0.5 * h * blep_post_step(wrap_frac) - natural;
                }

                if let Some(wfn) = next_wrap_frac {
                    let phase_at_next = wrap_phase(self.phasor.phase + (1.0 - wfn) * slave_dt);
                    let p_next = wrap_phase(self.params.sync_phase + slave_dt * wfn);
                    let h_next = 2.0 * (p_next - phase_at_next);
                    sample += 0.5 * h_next * blep_pre_step(wfn);
                }
                sample
            }
            SyncMode::Soft => {
                let dir_old = self.sync_direction;
                if master_wrapped {
                    self.sync_direction = -self.sync_direction;
                }
                let mut sample = self.generate_main_osc(freq * ratio * self.sync_direction, isr);

                if master_wrapped && aa_saw {
                    // Naïve saw slope per sample = 2·slave_dt·dir; flip → Δm = −4·slave_dt·dir_old.
                    let dm = -4.0 * slave_dt * dir_old;
                    sample += 0.5 * dm * blamp_post_kink(wrap_frac);
                }

                if let Some(wfn) = next_wrap_frac {
                    let dm_next = -4.0 * slave_dt * self.sync_direction;
                    sample += 0.5 * dm_next * blamp_pre_kink(wfn);
                }
                sample
            }
        }
    }

    fn generate_main_osc(&mut self, freq: f32, isr: f32) -> f32 {
        let pm = self.fm_phase_mod;
        match self.params.sound {
            Source::Tri => self.phasor.tri_shaped(freq, isr, &self.params.shape, pm) * 0.5,
            Source::Sine => self.phasor.sine_shaped(freq, isr, &self.params.shape, pm) * 0.5,
            Source::Saw => self.phasor.saw_shaped(freq, isr, &self.params.shape, pm) * 0.5,
            Source::Zaw => self.phasor.zaw_shaped(freq, isr, &self.params.shape, pm) * 0.5,
            Source::Pulse => {
                self.phasor
                    .pulse_shaped(freq, self.params.pw, isr, &self.params.shape, pm)
                    * 0.5
            }
            Source::Pulze => {
                self.phasor
                    .pulze_shaped(freq, self.params.pw, isr, &self.params.shape, pm)
                    * 0.5
            }
            Source::Osc => {
                let dt = freq * isr;
                let read = wrap_phase_any(self.phasor.phase + pm);
                let s = osc_morph_at(read, dt, self.params.wave, &self.params.shape);
                self.phasor.update(freq, isr);
                s * 0.5
            }
            Source::White => self.white() * 0.5,
            Source::Pink => {
                let w = self.white();
                self.pink_noise[0].next(w) * 0.5
            }
            Source::Brown => {
                let w = self.white();
                self.brown_noise.next(w) * 0.5
            }
            _ => 0.0,
        }
    }

    #[cfg(feature = "native")]
    #[allow(clippy::needless_range_loop)]
    fn run_wavetable(&mut self, freq: f32, isr: f32) -> [f32; CHANNELS] {
        // Compute modulated scan before borrowing registry_sample
        let scan = self.get_modulated_scan();
        let mut out = [0.0; CHANNELS];

        if let Some(ref rs) = self.registry_sample {
            let frame_count = rs.data.frame_count as f32;

            let cycle_len = if self.params.wt_cycle_len > 0 {
                self.params.wt_cycle_len as f32
            } else {
                frame_count
            };

            let num_cycles = (frame_count / cycle_len).floor().max(1.0);

            let phase = self.shape_phase(self.phasor.phase);

            let scan_pos = scan * (num_cycles - 1.0);
            let cycle_a = scan_pos.floor() as usize;
            let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
            let blend = scan_xfade(scan_pos.fract());

            let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
            let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

            let channels = rs.data.channels as usize;
            for c in 0..CHANNELS {
                let ch = c.min(channels - 1);
                let sample_a = rs.data.read_interpolated(pos_a, ch);
                let sample_b = rs.data.read_interpolated(pos_b, ch);
                out[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
            }

            self.phasor.update(freq, isr);
        }
        out
    }

    fn get_modulated_scan(&self) -> f32 {
        self.params.scan.clamp(0.0, 1.0)
    }

    #[cfg(not(feature = "native"))]
    fn run_wavetable_wasm(
        &mut self,
        freq: f32,
        isr: f32,
        pool: &[f32],
        samples: &[SampleInfo],
    ) -> [f32; CHANNELS] {
        let scan = self.get_modulated_scan();
        let mut out = [0.0; CHANNELS];

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

                let phase = self.shape_phase(self.phasor.phase);

                let scan_pos = scan * (num_cycles - 1.0);
                let cycle_a = scan_pos.floor() as usize;
                let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
                let blend = scan_xfade(scan_pos.fract());

                let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
                let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

                let frames = frame_count as usize;
                for c in 0..CHANNELS {
                    let ch = c.min(channels - 1);
                    let sample_a = read_interpolated(pool, offset, channels, frames, pos_a, ch);
                    let sample_b = read_interpolated(pool, offset, channels, frames, pos_b, ch);
                    out[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
                }

                self.phasor.update(freq, isr);
            }
        }
        out
    }

    #[cfg(not(feature = "native"))]
    fn run_wavetable_web(&mut self, freq: f32, isr: f32, web_pcm: &[f32]) -> [f32; CHANNELS] {
        let scan = self.get_modulated_scan();
        let mut out = [0.0; CHANNELS];

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

            let phase = self.shape_phase(self.phasor.phase);

            let scan_pos = scan * (num_cycles - 1.0);
            let cycle_a = scan_pos.floor() as usize;
            let cycle_b = (cycle_a + 1).min(num_cycles as usize - 1);
            let blend = scan_xfade(scan_pos.fract());

            let pos_a = (cycle_a as f32 * cycle_len) + (phase * cycle_len);
            let pos_b = (cycle_b as f32 * cycle_len) + (phase * cycle_len);

            let frames = frame_count as usize;
            for c in 0..CHANNELS {
                let ch = c.min(channels - 1);
                let sample_a = read_interpolated(web_pcm, offset, channels, frames, pos_a, ch);
                let sample_b = read_interpolated(web_pcm, offset, channels, frames, pos_b, ch);
                out[c] = (sample_a + blend * (sample_b - sample_a)) * 0.5;
            }

            self.phasor.update(freq, isr);
        }
        out
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
    if frames == 0 {
        return 0.0;
    }
    let center = pos.floor() as usize % frames;
    let frac = pos.fract();

    // Wavetable wraparound: cycle through `frames` for all 4 taps.
    let i0 = (center + frames - 1) % frames;
    let i1 = center;
    let i2 = (center + 1) % frames;
    let i3 = (center + 2) % frames;

    let read = |idx: usize| -> f32 {
        pool.get(offset + idx * channels + channel)
            .copied()
            .unwrap_or(0.0)
    };
    crate::dsp::hermite4(read(i0), read(i1), read(i2), read(i3), frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_sync_resets_main_phase_on_master_wrap() {
        let sr = 44_100.0_f32;
        let isr = 1.0 / sr;
        let freq = 100.0_f32;
        let ratio = 3.0_f32;

        let mut voice = Voice::default();
        voice.params.sound = Source::Saw;
        voice.params.sync_ratio = ratio;
        voice.params.sync_phase = 0.0;

        let samples_per_master_period = (sr / freq).ceil() as usize + 2;
        let mut wrap_count = 0usize;
        let mut prev_master = voice.sync_phasor.phase;
        let mut phase_after_wrap = f32::NAN;

        for _ in 0..samples_per_master_period {
            voice.run_single_osc(freq, isr);
            if voice.sync_phasor.phase < prev_master {
                wrap_count += 1;
                phase_after_wrap = voice.phasor.phase;
            }
            prev_master = voice.sync_phasor.phase;
        }

        assert_eq!(wrap_count, 1, "expected exactly one master wrap");
        // Phase is captured after the sample's advance: slave was reset to
        // (sync_phase + slave_dt * wrap_frac) and then advanced by slave_dt.
        let slave_dt = freq * ratio * isr;
        assert!(
            phase_after_wrap >= 0.0 && phase_after_wrap < 2.0 * slave_dt,
            "slave phase after sync should be within 2*slave_dt of 0, got {phase_after_wrap} (slave_dt={slave_dt})"
        );
    }

    #[test]
    fn hard_sync_ratio_one_is_no_op() {
        let sr = 44_100.0_f32;
        let isr = 1.0 / sr;
        let freq = 220.0_f32;

        let mut synced = Voice::default();
        synced.params.sound = Source::Saw;
        synced.params.sync_ratio = 1.0;

        let mut plain = Voice::default();
        plain.params.sound = Source::Saw;

        for _ in 0..256 {
            let sy = synced.run_single_osc(freq, isr);
            let pl = plain.run_single_osc(freq, isr);
            assert_eq!(sy.to_bits(), pl.to_bits());
        }
    }

    // With a post-step polyBLEP applied at each sync reset, the worst-case
    // sample-to-sample jump in the Saw output is bounded well below the raw
    // step amplitude (which can reach ±1.0 in ch[0] units after the 0.5 scale).
    #[test]
    fn hard_sync_saw_step_is_bounded() {
        let sr = 44_100.0_f32;
        let isr = 1.0 / sr;
        let freq = 110.0_f32;

        let mut voice = Voice::default();
        voice.params.sound = Source::Saw;
        voice.params.sync_ratio = 3.7;
        voice.params.sync_phase = 0.0;
        voice.params.sync_mode = SyncMode::Hard;

        // Natural saw wraps with 2-sample polyBLEP have a worst-case first-
        // difference of |slave_dt − 0.75| ≈ 0.74 (at τ≈0.5); that's the floor.
        // Without AA, sync wraps would add jumps close to |h|/2 ≈ 1.0 on top.
        // Bounding the overall max to ≲ 0.8 confirms sync jumps don't exceed
        // the natural-wrap baseline.
        let mut prev = 0.0_f32;
        let mut max_jump = 0.0_f32;
        for i in 0..2048 {
            let y = voice.run_single_osc(freq, isr);
            if i > 0 {
                let d = (y - prev).abs();
                if d > max_jump {
                    max_jump = d;
                }
            }
            prev = y;
        }
        assert!(
            max_jump < 0.8,
            "hard-sync saw first-difference should stay at natural-wrap baseline, got {max_jump}"
        );
    }

    // PolyBLAMP smooths the direction-reversal kink. The dominant 2nd-difference
    // contribution is still the natural saw wrap (now band-limited in both
    // directions after the negative-`dt` fix in `poly_blep`). Without AA for
    // reversed direction, the 2nd difference is ≳1.5; with AA it stays ≲1.0.
    #[test]
    fn soft_sync_saw_kink_is_bounded() {
        let sr = 44_100.0_f32;
        let isr = 1.0 / sr;
        let freq = 110.0_f32;

        let mut voice = Voice::default();
        voice.params.sound = Source::Saw;
        voice.params.sync_ratio = 3.7;
        voice.params.sync_mode = SyncMode::Soft;

        let mut y_prev = 0.0_f32;
        let mut y_prev2 = 0.0_f32;
        let mut max_2nd = 0.0_f32;
        for i in 0..2048 {
            let y = voice.run_single_osc(freq, isr);
            if i >= 2 {
                let d2 = (y - 2.0 * y_prev + y_prev2).abs();
                if d2 > max_2nd {
                    max_2nd = d2;
                }
            }
            y_prev2 = y_prev;
            y_prev = y;
        }
        assert!(
            max_2nd < 1.0,
            "soft-sync saw second-difference should be bounded, got {max_2nd}"
        );
    }
}
