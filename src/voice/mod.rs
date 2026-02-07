//! Voice - the core synthesis unit.

mod modulation;
mod params;
mod source;

pub use modulation::{ModChain, ParamId, ParamMod};
pub use params::VoiceParams;

use std::f32::consts::PI;

use crate::dsp::{cosf, exp2f, sinf, Adsr, BrownNoise, Phasor, PinkNoise, SvfMode, SvfState};
use crate::effects::{
    crush, distort, fold, wrap, Chorus, Coarse, Eq, Flanger, Haas, LadderFilter, LadderMode, Lag,
    Phaser, Tilt,
};
use crate::plaits::PlaitsEngine;
#[cfg(feature = "native")]
use crate::sampling::RegistrySample;
use crate::sampling::WebSampleSource;
#[cfg(not(feature = "native"))]
use crate::sampling::{FileSource, SampleInfo};
use crate::types::{FilterSlope, BLOCK_SIZE, CHANNELS};

pub const MAX_PARAM_MODS: usize = 8;

pub struct Voice {
    pub params: VoiceParams,
    pub phasor: Phasor,
    pub sub_phasor: Phasor,
    pub spread_phasors: [Phasor; 7],
    pub adsr: Adsr,
    pub lp_adsr: Adsr,
    pub hp_adsr: Adsr,
    pub bp_adsr: Adsr,
    pub lp: SvfState,
    pub hp: SvfState,
    pub bp: SvfState,
    // Modulation
    pub pitch_adsr: Adsr,
    pub fm_adsr: Adsr,
    pub vib_lfo: Phasor,
    pub fm_phasor: Phasor,
    pub fm2_phasor: Phasor,
    pub fm_fb_prev: f32,
    pub fm_fb_prev2: f32,
    pub am_lfo: Phasor,
    pub rm_lfo: Phasor,
    pub glide_lag: Lag,
    pub current_freq: f32,
    // Noise
    pub pink_noise: PinkNoise,
    pub brown_noise: BrownNoise,
    // Sample playback (WASM legacy pool)
    #[cfg(not(feature = "native"))]
    pub file_source: Option<FileSource>,
    // Sample playback (native lock-free registry)
    #[cfg(feature = "native")]
    pub registry_sample: Option<RegistrySample>,
    // Sample playback (web)
    pub web_sample: Option<WebSampleSource>,
    // Effects
    pub phaser: Phaser,
    pub flanger: Flanger,
    pub chorus: Chorus,
    pub coarse: Coarse,
    pub eq: Eq,
    pub tilt: Tilt,
    pub haas: Haas,
    pub ladder_lp: LadderFilter,
    pub ladder_hp: LadderFilter,
    pub ladder_bp: LadderFilter,

    // Inline parameter modulation
    pub param_mods: [(ParamId, ParamMod); MAX_PARAM_MODS],
    pub param_mod_count: u8,

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
            lp: SvfState::default(),
            hp: SvfState::default(),
            bp: SvfState::default(),
            pitch_adsr: Adsr::default(),
            fm_adsr: Adsr::default(),
            vib_lfo: Phasor::default(),
            fm_phasor: Phasor::default(),
            fm2_phasor: Phasor::default(),
            fm_fb_prev: 0.0,
            fm_fb_prev2: 0.0,
            am_lfo: Phasor::default(),
            rm_lfo: Phasor::default(),
            glide_lag: Lag::default(),
            current_freq: 330.0,
            pink_noise: PinkNoise::default(),
            brown_noise: BrownNoise::default(),
            #[cfg(not(feature = "native"))]
            file_source: None,
            #[cfg(feature = "native")]
            registry_sample: None,
            web_sample: None,
            phaser: Phaser::default(),
            flanger: Flanger::default(),
            chorus: Chorus::default(),
            coarse: Coarse::default(),
            eq: Eq::default(),
            tilt: Tilt::default(),
            haas: Haas::default(),
            ladder_lp: LadderFilter::default(),
            ladder_hp: LadderFilter::default(),
            ladder_bp: LadderFilter::default(),
            param_mods: [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS],
            param_mod_count: 0,
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
            fm2_phasor: self.fm2_phasor,
            fm_fb_prev: self.fm_fb_prev,
            fm_fb_prev2: self.fm_fb_prev2,
            am_lfo: self.am_lfo,
            rm_lfo: self.rm_lfo,
            glide_lag: self.glide_lag,
            current_freq: self.current_freq,
            pink_noise: self.pink_noise,
            brown_noise: self.brown_noise,
            #[cfg(not(feature = "native"))]
            file_source: self.file_source,
            #[cfg(feature = "native")]
            registry_sample: self.registry_sample.clone(),
            web_sample: self.web_sample,
            phaser: self.phaser,
            flanger: self.flanger,
            chorus: self.chorus,
            coarse: self.coarse,
            eq: self.eq,
            tilt: self.tilt,
            haas: self.haas,
            ladder_lp: self.ladder_lp,
            ladder_hp: self.ladder_hp,
            ladder_bp: self.ladder_bp,
            param_mods: self.param_mods,
            param_mod_count: self.param_mod_count,
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
        self.seed = modulation::lcg(self.seed);
        ((self.seed >> 16) & 0x7fff) as f32 / 32767.0
    }

    pub(super) fn white(&mut self) -> f32 {
        self.rand() * 2.0 - 1.0
    }

    pub fn set_mod(&mut self, id: ParamId, chain: ModChain) {
        for i in 0..self.param_mod_count as usize {
            if self.param_mods[i].0 == id {
                self.param_mods[i].1 = ParamMod::new(chain, self.seed);
                self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
                return;
            }
        }
        if (self.param_mod_count as usize) < MAX_PARAM_MODS {
            let i = self.param_mod_count as usize;
            self.param_mods[i] = (id, ParamMod::new(chain, self.seed));
            self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
            self.param_mod_count += 1;
        }
    }

    fn apply_mods(&mut self, isr: f32) {
        for i in 0..self.param_mod_count as usize {
            let (id, ref mut m) = self.param_mods[i];
            let val = m.tick(isr);
            self.write_param(id, val);
        }
    }

    fn write_param(&mut self, id: ParamId, val: f32) {
        match id {
            ParamId::Freq => self.params.freq = val,
            ParamId::Gain => self.params.gain = val,
            ParamId::Postgain => self.params.postgain = val,
            ParamId::Pan => self.params.pan = val,
            ParamId::Speed => self.params.speed = val,
            ParamId::Detune => self.params.detune = val,
            ParamId::Pw => self.params.pw = val,
            ParamId::Sub => self.params.sub = val,
            ParamId::Harmonics => self.params.harmonics = val,
            ParamId::Timbre => self.params.timbre = val,
            ParamId::Morph => self.params.morph = val,
            ParamId::Scan => self.params.scan = val,
            ParamId::Lpf => self.params.lpf = Some(val),
            ParamId::Lpq => self.params.lpq = val,
            ParamId::Hpf => self.params.hpf = Some(val),
            ParamId::Hpq => self.params.hpq = val,
            ParamId::Bpf => self.params.bpf = Some(val),
            ParamId::Bpq => self.params.bpq = val,
            ParamId::Llpf => self.params.llpf = Some(val),
            ParamId::Llpq => self.params.llpq = val,
            ParamId::Lhpf => self.params.lhpf = Some(val),
            ParamId::Lhpq => self.params.lhpq = val,
            ParamId::Lbpf => self.params.lbpf = Some(val),
            ParamId::Lbpq => self.params.lbpq = val,
            ParamId::Fm => self.params.fm = val,
            ParamId::Fmh => self.params.fmh = val,
            ParamId::Fm2 => self.params.fm2 = val,
            ParamId::Fm2h => self.params.fm2h = val,
            ParamId::Fmfb => self.params.fmfb = val,
            ParamId::Am => self.params.am = val,
            ParamId::Amdepth => self.params.amdepth = val,
            ParamId::Rm => self.params.rm = val,
            ParamId::Rmdepth => self.params.rmdepth = val,
            ParamId::Vib => self.params.vib = val,
            ParamId::Vibmod => self.params.vibmod = val,
            ParamId::Phaser => self.params.phaser = val,
            ParamId::Phaserdepth => self.params.phaserdepth = val,
            ParamId::Phasersweep => self.params.phasersweep = val,
            ParamId::Phasercenter => self.params.phasercenter = val,
            ParamId::Flanger => self.params.flanger = val,
            ParamId::Flangerdepth => self.params.flangerdepth = val,
            ParamId::Flangerfeedback => self.params.flangerfeedback = val,
            ParamId::Chorus => self.params.chorus = val,
            ParamId::Chorusdepth => self.params.chorusdepth = val,
            ParamId::Chorusdelay => self.params.chorusdelay = val,
            ParamId::Fold => self.params.fold = Some(val),
            ParamId::Crush => self.params.crush = Some(val),
            ParamId::Coarse => self.params.coarse = Some(val),
            ParamId::Distort => self.params.distort = Some(val),
            ParamId::Wrap => self.params.wrap = Some(val),
            ParamId::Eqlo => self.params.eqlo = val,
            ParamId::Eqmid => self.params.eqmid = val,
            ParamId::Eqhi => self.params.eqhi = val,
            ParamId::Tilt => self.params.tilt = val,
            ParamId::Width => self.params.width = val,
            ParamId::Haas => self.params.haas = val,
            ParamId::Delay => self.params.delay = val,
            ParamId::Verb => self.params.verb = val,
            ParamId::Comb => self.params.comb = val,
            ParamId::Feedback => self.params.feedback = val,
        }
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

        // FM synthesis (3-operator)
        if self.params.fm > 0.0 || self.params.fm2 > 0.0 {
            let env_scale = if self.params.fm_env_active {
                let env = self.fm_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.fma,
                    self.params.fmd,
                    self.params.fms,
                    self.params.fmr,
                );
                self.params.fme * env + 1.0
            } else {
                1.0
            };

            let fm1 = self.params.fm * env_scale;
            let fm2 = self.params.fm2 * env_scale;
            let shape = self.params.fmshape;
            let fb = (self.fm_fb_prev + self.fm_fb_prev2) * 0.5 * self.params.fmfb;

            if fm2 > 0.0 {
                match self.params.fmalgo {
                    // Cascade: fm2 -> fm1 -> carrier
                    0 => {
                        let mod2_freq = freq * self.params.fm2h;
                        let mod2 = self.fm2_phasor.lfo(shape, mod2_freq + fb * mod2_freq, isr);
                        self.fm_fb_prev2 = self.fm_fb_prev;
                        self.fm_fb_prev = mod2;
                        let mod1_base = freq * self.params.fmh;
                        let mod1_freq = mod1_base + mod2 * mod2_freq * fm2;
                        let mod1 = self.fm_phasor.lfo(shape, mod1_freq, isr);
                        freq += mod1 * mod1_base * fm1;
                    }
                    // Parallel: fm1 + fm2 both modulate carrier
                    1 => {
                        let mf1 = freq * self.params.fmh;
                        freq += self.fm_phasor.lfo(shape, mf1, isr) * mf1 * fm1;
                        let mf2 = freq * self.params.fm2h;
                        let mod2 = self.fm2_phasor.lfo(shape, mf2 + fb * mf2, isr);
                        self.fm_fb_prev2 = self.fm_fb_prev;
                        self.fm_fb_prev = mod2;
                        freq += mod2 * mf2 * fm2;
                    }
                    // Branch: fm2 -> fm1 -> carrier AND fm2 -> carrier
                    _ => {
                        let mod2_freq = freq * self.params.fm2h;
                        let mod2 = self.fm2_phasor.lfo(shape, mod2_freq + fb * mod2_freq, isr);
                        self.fm_fb_prev2 = self.fm_fb_prev;
                        self.fm_fb_prev = mod2;
                        let mod1_base = freq * self.params.fmh;
                        let mod1_freq = mod1_base + mod2 * mod2_freq * fm2;
                        let mod1 = self.fm_phasor.lfo(shape, mod1_freq, isr);
                        freq += mod1 * mod1_base * fm1;
                        freq += mod2 * mod2_freq * fm2;
                    }
                }
            } else {
                let mod1_freq = freq * self.params.fmh;
                let mod1 = self.fm_phasor.lfo(shape, mod1_freq + fb * mod1_freq, isr);
                self.fm_fb_prev2 = self.fm_fb_prev;
                self.fm_fb_prev = mod1;
                freq += mod1 * mod1_freq * fm1;
            }
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

    #[cfg(feature = "native")]
    pub fn process(
        &mut self,
        isr: f32,
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

        if self.param_mod_count > 0 {
            self.apply_mods(isr);
        }
        let freq = self.compute_freq(isr);
        if !self.run_source(freq, isr, web_pcm, sample_idx, live_input) {
            return false;
        }

        self.apply_filters_and_effects(env);
        true
    }

    #[cfg(not(feature = "native"))]
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

        if self.param_mod_count > 0 {
            self.apply_mods(isr);
        }
        let freq = self.compute_freq(isr);
        if !self.run_source(freq, isr, pool, samples, web_pcm, sample_idx, live_input) {
            return false;
        }

        self.apply_filters_and_effects(env);
        true
    }

    fn apply_filters_and_effects(&mut self, env: f32) {
        let isr = 1.0 / self.sr;
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
            self.ch[0] =
                self.lp
                    .process(self.ch[0], SvfMode::Lp, self.params.lpq, num_stages, self.sr);
        }
        if self.params.hpf.is_some() {
            self.ch[0] =
                self.hp
                    .process(self.ch[0], SvfMode::Hp, self.params.hpq, num_stages, self.sr);
        }
        if self.params.bpf.is_some() {
            self.ch[0] =
                self.bp
                    .process(self.ch[0], SvfMode::Bp, self.params.bpq, num_stages, self.sr);
        }

        // Ladder filters (compute envelope independently from biquad filters)
        if let Some(llpf) = self.params.llpf {
            let mut cutoff = llpf;
            if self.params.lp_env_active {
                let env = self.lp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.lpa,
                    self.params.lpd,
                    self.params.lps,
                    self.params.lpr,
                );
                cutoff = self.params.lpe * env * llpf + llpf;
            }
            self.ch[0] = self.ladder_lp.process(
                self.ch[0],
                cutoff,
                self.params.llpq,
                LadderMode::Lp,
                self.sr,
            );
        }
        if let Some(lhpf) = self.params.lhpf {
            let mut cutoff = lhpf;
            if self.params.hp_env_active {
                let env = self.hp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.hpa,
                    self.params.hpd,
                    self.params.hps,
                    self.params.hpr,
                );
                cutoff = self.params.hpe * env * lhpf + lhpf;
            }
            self.ch[0] = self.ladder_hp.process(
                self.ch[0],
                cutoff,
                self.params.lhpq,
                LadderMode::Hp,
                self.sr,
            );
        }
        if let Some(lbpf) = self.params.lbpf {
            let mut cutoff = lbpf;
            if self.params.bp_env_active {
                let env = self.bp_adsr.update(
                    self.time,
                    self.params.gate,
                    self.params.bpa,
                    self.params.bpd,
                    self.params.bps,
                    self.params.bpr,
                );
                cutoff = self.params.bpe * env * lbpf + lbpf;
            }
            self.ch[0] = self.ladder_bp.process(
                self.ch[0],
                cutoff,
                self.params.lbpq,
                LadderMode::Bp,
                self.sr,
            );
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

        // EQ
        if self.params.eqlo != 0.0 || self.params.eqmid != 0.0 || self.params.eqhi != 0.0 {
            self.ch[0] = self.eq.process(
                self.ch[0],
                self.params.eqlo,
                self.params.eqmid,
                self.params.eqhi,
                self.sr,
            );
        }

        // Tilt
        if self.params.tilt != 0.0 {
            self.ch[0] = self.tilt.process(self.ch[0], self.params.tilt, self.sr);
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

        // Stereo width (mid-side matrix)
        if self.params.width != 1.0 {
            let mid = (self.ch[0] + self.ch[1]) * 0.5;
            let side = (self.ch[0] - self.ch[1]) * 0.5;
            let w = self.params.width.max(0.0);
            self.ch[0] = mid + side * w;
            self.ch[1] = mid - side * w;
        }

        // Haas (delay right channel)
        if self.params.haas > 0.0 {
            self.ch[1] = self.haas.process(self.ch[1], self.params.haas, self.sr);
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
    }
}
