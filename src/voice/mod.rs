//! Voice - the core synthesis unit.

mod params;
mod source;

pub use params::VoiceParams;

use std::f32::consts::PI;

use crate::effects::{crush, distort, fold, wrap, Chorus, Coarse, Flanger, Lag, LadderFilter, LadderMode, Phaser};
use crate::envelope::Adsr;
use crate::fastmath::{cosf, exp2f, sinf};
use crate::filter::FilterState;
use crate::noise::{BrownNoise, PinkNoise};
use crate::oscillator::Phasor;
use crate::plaits::PlaitsEngine;
use crate::sample::{FileSource, SampleInfo, WebSampleSource};
use crate::types::{FilterSlope, FilterType, BLOCK_SIZE, CHANNELS};

fn apply_filter(
    signal: f32,
    filter: &mut FilterState,
    ftype: FilterType,
    q: f32,
    num_stages: usize,
    sr: f32,
) -> f32 {
    let mut out = signal;
    for stage in 0..num_stages {
        out = filter.biquads[stage].process(out, ftype, filter.cutoff, q, sr);
    }
    out
}

pub struct Voice {
    pub params: VoiceParams,
    pub phasor: Phasor,
    pub sub_phasor: Phasor,
    pub spread_phasors: [Phasor; 7],
    pub adsr: Adsr,
    pub lp_adsr: Adsr,
    pub hp_adsr: Adsr,
    pub bp_adsr: Adsr,
    pub lp: FilterState,
    pub hp: FilterState,
    pub bp: FilterState,
    // Modulation
    pub pitch_adsr: Adsr,
    pub fm_adsr: Adsr,
    pub vib_lfo: Phasor,
    pub fm_phasor: Phasor,
    pub am_lfo: Phasor,
    pub rm_lfo: Phasor,
    pub glide_lag: Lag,
    pub current_freq: f32,
    // Noise
    pub pink_noise: PinkNoise,
    pub brown_noise: BrownNoise,
    // Sample playback (native)
    pub file_source: Option<FileSource>,
    // Sample playback (web)
    pub web_sample: Option<WebSampleSource>,
    // Effects
    pub phaser: Phaser,
    pub flanger: Flanger,
    pub chorus: Chorus,
    pub coarse: Coarse,
    pub ladder_lp: LadderFilter,
    pub ladder_hp: LadderFilter,
    pub ladder_bp: LadderFilter,

    pub time: f32,
    pub ch: [f32; CHANNELS],
    pub spread_side: f32,
    pub sr: f32,
    pub lag_unit: f32,
    pub(super) seed: u32,

    // Plaits engines
    pub(super) plaits_engine: Option<PlaitsEngine>,
    pub(super) plaits_out: [f32; BLOCK_SIZE],
    pub(super) plaits_aux: [f32; BLOCK_SIZE],
    pub(super) plaits_idx: usize,
    pub(super) plaits_prev_gate: bool,
}

impl Default for Voice {
    fn default() -> Self {
        let sr = 44100.0;
        Self {
            params: VoiceParams::default(),
            phasor: Phasor::default(),
            sub_phasor: Phasor::default(),
            spread_phasors: std::array::from_fn(|i| {
                let mut p = Phasor::default();
                p.phase = i as f32 / 7.0;
                p
            }),
            adsr: Adsr::default(),
            lp_adsr: Adsr::default(),
            hp_adsr: Adsr::default(),
            bp_adsr: Adsr::default(),
            lp: FilterState::default(),
            hp: FilterState::default(),
            bp: FilterState::default(),
            pitch_adsr: Adsr::default(),
            fm_adsr: Adsr::default(),
            vib_lfo: Phasor::default(),
            fm_phasor: Phasor::default(),
            am_lfo: Phasor::default(),
            rm_lfo: Phasor::default(),
            glide_lag: Lag::default(),
            current_freq: 330.0,
            pink_noise: PinkNoise::default(),
            brown_noise: BrownNoise::default(),
            file_source: None,
            web_sample: None,
            phaser: Phaser::default(),
            flanger: Flanger::default(),
            chorus: Chorus::default(),
            coarse: Coarse::default(),
            ladder_lp: LadderFilter::default(),
            ladder_hp: LadderFilter::default(),
            ladder_bp: LadderFilter::default(),
            time: 0.0,
            ch: [0.0; CHANNELS],
            spread_side: 0.0,
            sr,
            lag_unit: sr / 10.0,
            seed: 123456789,
            plaits_engine: None,
            plaits_out: [0.0; BLOCK_SIZE],
            plaits_aux: [0.0; BLOCK_SIZE],
            plaits_idx: BLOCK_SIZE,
            plaits_prev_gate: false,
        }
    }
}

impl Clone for Voice {
    fn clone(&self) -> Self {
        Self {
            params: self.params,
            phasor: self.phasor,
            sub_phasor: self.sub_phasor,
            spread_phasors: self.spread_phasors,
            adsr: self.adsr,
            lp_adsr: self.lp_adsr,
            hp_adsr: self.hp_adsr,
            bp_adsr: self.bp_adsr,
            lp: self.lp,
            hp: self.hp,
            bp: self.bp,
            pitch_adsr: self.pitch_adsr,
            fm_adsr: self.fm_adsr,
            vib_lfo: self.vib_lfo,
            fm_phasor: self.fm_phasor,
            am_lfo: self.am_lfo,
            rm_lfo: self.rm_lfo,
            glide_lag: self.glide_lag,
            current_freq: self.current_freq,
            pink_noise: self.pink_noise,
            brown_noise: self.brown_noise,
            file_source: self.file_source,
            web_sample: self.web_sample,
            phaser: self.phaser,
            flanger: self.flanger,
            chorus: self.chorus,
            coarse: self.coarse,
            ladder_lp: self.ladder_lp,
            ladder_hp: self.ladder_hp,
            ladder_bp: self.ladder_bp,
            time: self.time,
            ch: self.ch,
            spread_side: self.spread_side,
            sr: self.sr,
            lag_unit: self.lag_unit,
            seed: self.seed,
            plaits_engine: None,
            plaits_out: [0.0; BLOCK_SIZE],
            plaits_aux: [0.0; BLOCK_SIZE],
            plaits_idx: BLOCK_SIZE,
            plaits_prev_gate: false,
        }
    }
}

impl Voice {
    pub(super) fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) & 0x7fff) as f32 / 32767.0
    }

    pub(super) fn white(&mut self) -> f32 {
        self.rand() * 2.0 - 1.0
    }

    fn compute_freq(&mut self, isr: f32) -> f32 {
        let mut freq = self.params.freq;

        // Detune (cents offset)
        if self.params.detune != 0.0 {
            freq *= exp2f(self.params.detune / 1200.0);
        }

        // Speed multiplier
        freq *= self.params.speed;

        // Glide
        if let Some(glide_time) = self.params.glide {
            freq = self.glide_lag.update(freq, glide_time, self.lag_unit);
        }

        // FM synthesis
        if self.params.fm > 0.0 {
            let mut fm_amount = self.params.fm;
            if self.params.fm_env_active {
                let env = self.fm_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.fma,
                    self.params.fmd,
                    self.params.fms,
                    self.params.fmr,
                );
                fm_amount = self.params.fme * env * fm_amount + fm_amount;
            }
            let mod_freq = freq * self.params.fmh;
            let mod_gain = mod_freq * fm_amount;
            let modulator = self.fm_phasor.lfo(self.params.fmshape, mod_freq, isr);
            freq += modulator * mod_gain;
        }

        // Pitch envelope
        if self.params.pitch_env_active && self.params.penv != 0.0 {
            let env = self.pitch_adsr.update(
                self.time,
                1.0,
                self.params.patt,
                self.params.pdec,
                self.params.psus,
                self.params.prel,
            );
            let env_adj = if self.params.psus == 1.0 {
                env - 1.0
            } else {
                env
            };
            freq *= exp2f(env_adj * self.params.penv / 12.0);
        }

        // Vibrato
        if self.params.vib > 0.0 && self.params.vibmod > 0.0 {
            let mod_val = self.vib_lfo.lfo(self.params.vibshape, self.params.vib, isr);
            freq *= exp2f(mod_val * self.params.vibmod / 12.0);
        }

        self.current_freq = freq;
        freq
    }

    fn num_stages(&self) -> usize {
        match self.params.ftype {
            FilterSlope::Db12 => 1,
            FilterSlope::Db24 => 2,
            FilterSlope::Db48 => 4,
        }
    }

    pub fn process(
        &mut self,
        isr: f32,
        pool: &[f32],
        samples: &[SampleInfo],
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
    ) -> bool {
        let env = self.adsr.update(
            self.time,
            self.params.gate,
            self.params.attack,
            self.params.decay,
            self.params.sustain,
            self.params.release,
        );
        if self.adsr.is_off() {
            return false;
        }

        let freq = self.compute_freq(isr);
        if !self.run_source(freq, isr, pool, samples, web_pcm, sample_idx, live_input) {
            return false;
        }

        // Update filter envelopes
        if let Some(lpf) = self.params.lpf {
            self.lp.cutoff = lpf;
            if self.params.lp_env_active {
                let lp_env = self.lp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.lpa,
                    self.params.lpd,
                    self.params.lps,
                    self.params.lpr,
                );
                self.lp.cutoff = self.params.lpe * lp_env * lpf + lpf;
            }
        }
        if let Some(hpf) = self.params.hpf {
            self.hp.cutoff = hpf;
            if self.params.hp_env_active {
                let hp_env = self.hp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.hpa,
                    self.params.hpd,
                    self.params.hps,
                    self.params.hpr,
                );
                self.hp.cutoff = self.params.hpe * hp_env * hpf + hpf;
            }
        }
        if let Some(bpf) = self.params.bpf {
            self.bp.cutoff = bpf;
            if self.params.bp_env_active {
                let bp_env = self.bp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.bpa,
                    self.params.bpd,
                    self.params.bps,
                    self.params.bpr,
                );
                self.bp.cutoff = self.params.bpe * bp_env * bpf + bpf;
            }
        }

        // Pre-filter gain
        self.ch[0] *= self.params.gain * self.params.velocity;

        // Apply filters (LP -> HP -> BP)
        let num_stages = self.num_stages();
        if self.params.lpf.is_some() {
            self.ch[0] = apply_filter(
                self.ch[0],
                &mut self.lp,
                FilterType::Lowpass,
                self.params.lpq,
                num_stages,
                self.sr,
            );
        }
        if self.params.hpf.is_some() {
            self.ch[0] = apply_filter(
                self.ch[0],
                &mut self.hp,
                FilterType::Highpass,
                self.params.hpq,
                num_stages,
                self.sr,
            );
        }
        if self.params.bpf.is_some() {
            self.ch[0] = apply_filter(
                self.ch[0],
                &mut self.bp,
                FilterType::Bandpass,
                self.params.bpq,
                num_stages,
                self.sr,
            );
        }

        // Ladder filters (compute envelope independently from biquad filters)
        if let Some(llpf) = self.params.llpf {
            let mut cutoff = llpf;
            if self.params.lp_env_active {
                let env = self.lp_adsr.update(
                    self.time, self.params.gate,
                    self.params.lpa, self.params.lpd, self.params.lps, self.params.lpr,
                );
                cutoff = self.params.lpe * env * llpf + llpf;
            }
            self.ch[0] = self.ladder_lp.process(self.ch[0], cutoff, self.params.llpq, LadderMode::Lp, self.sr);
        }
        if let Some(lhpf) = self.params.lhpf {
            let mut cutoff = lhpf;
            if self.params.hp_env_active {
                let env = self.hp_adsr.update(
                    self.time, self.params.gate,
                    self.params.hpa, self.params.hpd, self.params.hps, self.params.hpr,
                );
                cutoff = self.params.hpe * env * lhpf + lhpf;
            }
            self.ch[0] = self.ladder_hp.process(self.ch[0], cutoff, self.params.lhpq, LadderMode::Hp, self.sr);
        }
        if let Some(lbpf) = self.params.lbpf {
            let mut cutoff = lbpf;
            if self.params.bp_env_active {
                let env = self.bp_adsr.update(
                    self.time, self.params.gate,
                    self.params.bpa, self.params.bpd, self.params.bps, self.params.bpr,
                );
                cutoff = self.params.bpe * env * lbpf + lbpf;
            }
            self.ch[0] = self.ladder_bp.process(self.ch[0], cutoff, self.params.lbpq, LadderMode::Bp, self.sr);
        }

        // Distortion effects
        if let Some(coarse_factor) = self.params.coarse {
            self.ch[0] = self.coarse.process(self.ch[0], coarse_factor);
        }
        if let Some(crush_bits) = self.params.crush {
            self.ch[0] = crush(self.ch[0], crush_bits);
        }
        if let Some(fold_amount) = self.params.fold {
            self.ch[0] = fold(self.ch[0], fold_amount);
        }
        if let Some(wrap_amount) = self.params.wrap {
            self.ch[0] = wrap(self.ch[0], wrap_amount);
        }
        if let Some(dist_amount) = self.params.distort {
            self.ch[0] = distort(self.ch[0], dist_amount, self.params.distortvol);
        }

        // AM modulation
        if self.params.am > 0.0 {
            let modulator = self.am_lfo.lfo(self.params.amshape, self.params.am, isr);
            let depth = self.params.amdepth.clamp(0.0, 1.0);
            self.ch[0] *= 1.0 + modulator * depth;
        }

        // Ring modulation
        if self.params.rm > 0.0 {
            let modulator = self.rm_lfo.lfo(self.params.rmshape, self.params.rm, isr);
            let depth = self.params.rmdepth.clamp(0.0, 1.0);
            self.ch[0] *= (1.0 - depth) + modulator * depth;
        }

        // Phaser
        if self.params.phaser > 0.0 {
            self.ch[0] = self.phaser.process(
                self.ch[0],
                self.params.phaser,
                self.params.phaserdepth,
                self.params.phasercenter,
                self.params.phasersweep,
                self.sr,
                isr,
            );
        }

        // Flanger
        if self.params.flanger > 0.0 {
            self.ch[0] = self.flanger.process(
                self.ch[0],
                self.params.flanger,
                self.params.flangerdepth,
                self.params.flangerfeedback,
                self.sr,
                isr,
            );
        }

        // Apply gain envelope and postgain
        self.ch[0] *= env * self.params.postgain;

        // Restore stereo for spread mode
        if self.params.spread > 0.0 {
            let side = self.spread_side * env * self.params.postgain;
            self.ch[1] = self.ch[0] - side;
            self.ch[0] += side;
        } else {
            self.ch[1] = self.ch[0];
        }

        // Chorus
        if self.params.chorus > 0.0 {
            let stereo = self.chorus.process(
                self.ch[0],
                self.ch[1],
                self.params.chorus,
                self.params.chorusdepth,
                self.params.chorusdelay,
                self.sr,
                isr,
            );
            self.ch[0] = stereo[0];
            self.ch[1] = stereo[1];
        }

        // Panning
        if self.params.pan != 0.5 {
            let pan_pos = self.params.pan * PI / 2.0;
            self.ch[0] *= cosf(pan_pos);
            self.ch[1] *= sinf(pan_pos);
        }

        self.time += isr;
        if let Some(dur) = self.params.duration {
            if dur > 0.0 && self.time > dur {
                self.params.gate = 0.0;
            }
        }
        true
    }
}
