//! Voice - the core synthesis unit.

mod drums;
pub mod modulation;
mod params;
mod source;

pub use modulation::{ModChain, ParamId, ParamMod};
pub use params::VoiceParams;

use std::f32::consts::PI;

use crate::dsp::{cosf, exp2f, sinf, BrownNoise, Dahdsr, Phasor, PinkNoise, SvfMode, SvfState};
use crate::effects::{
    crush, distort, fold, wrap, Chorus, Coarse, Eq, Flanger, Haas, LadderFilter, LadderMode,
    Phaser, Smear, Tilt,
};
#[cfg(feature = "native")]
use crate::sampling::RegistrySample;
#[cfg(feature = "native")]
use crate::sampling::StretchState;
use crate::sampling::WebSampleSource;
#[cfg(not(feature = "native"))]
use crate::sampling::{FileSource, SampleInfo};
use crate::types::CHANNELS;

pub const MAX_PARAM_MODS: usize = 15;
pub(crate) const MAX_ADDITIVE_PARTIALS: usize = 32;

#[derive(Clone, Copy)]
pub(crate) struct AdditiveCache {
    pub ratios: [f32; MAX_ADDITIVE_PARTIALS],
    pub amps: [f32; MAX_ADDITIVE_PARTIALS],
    pub norm_prefix: [f32; MAX_ADDITIVE_PARTIALS],
    pub active_count: u8,
    pub tail_weight: f32,
    pub valid: bool,
}

impl Default for AdditiveCache {
    fn default() -> Self {
        Self {
            ratios: [0.0; MAX_ADDITIVE_PARTIALS],
            amps: [0.0; MAX_ADDITIVE_PARTIALS],
            norm_prefix: [0.0; MAX_ADDITIVE_PARTIALS],
            active_count: 0,
            tail_weight: 0.0,
            valid: false,
        }
    }
}

pub struct Voice {
    pub params: VoiceParams,
    pub phasor: Phasor,
    pub sub_phasor: Phasor,
    pub spread_phasors: [Phasor; 7],
    pub dahdsr: Dahdsr,
    pub lp: [SvfState; CHANNELS],
    pub hp: [SvfState; CHANNELS],
    pub bp: [SvfState; CHANNELS],
    pub vib_lfo: Phasor,
    pub fm_phasor: Phasor,
    pub fm2_phasor: Phasor,
    pub fm_fb_prev: f32,
    pub fm_fb_prev2: f32,
    pub am_lfo: Phasor,
    pub rm_lfo: Phasor,
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
    #[cfg(feature = "native")]
    pub registry_sample_b: Option<RegistrySample>,
    pub sample_blend: f32,
    #[cfg(feature = "native")]
    pub stretch: StretchState,
    // Sample playback (web)
    pub web_sample: Option<WebSampleSource>,
    // Effects
    pub phaser: [Phaser; CHANNELS],
    pub flanger: Option<Box<[Flanger; CHANNELS]>>,
    pub smear: [Smear; CHANNELS],
    pub chorus: Option<Box<Chorus>>,
    pub coarse: [Coarse; CHANNELS],
    pub eq: [Eq; CHANNELS],
    pub tilt: [Tilt; CHANNELS],
    pub haas: Option<Box<Haas>>,
    pub ladder_lp: [LadderFilter; CHANNELS],
    pub ladder_hp: [LadderFilter; CHANNELS],
    pub ladder_bp: [LadderFilter; CHANNELS],

    // Inline parameter modulation
    pub param_mods: [(ParamId, ParamMod); MAX_PARAM_MODS],
    pub param_mod_count: u8,

    pub triggered: bool,
    pub time: f32,
    pub ch: [f32; CHANNELS],
    pub nch: usize,
    pub spread_side: f32,
    pub spread_cache_value: f32,
    pub spread_detune_ratios: [f32; 3],
    pub(crate) additive_cache: AdditiveCache,
    pub(crate) shape_active: bool,
    pub sr: f32,
    pub seed: u32,

    // Drum synthesis filter
    pub(super) drum_svf: SvfState,
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
            dahdsr: Dahdsr::default(),
            lp: [SvfState::default(); CHANNELS],
            hp: [SvfState::default(); CHANNELS],
            bp: [SvfState::default(); CHANNELS],
            vib_lfo: Phasor::default(),
            fm_phasor: Phasor::default(),
            fm2_phasor: Phasor::default(),
            fm_fb_prev: 0.0,
            fm_fb_prev2: 0.0,
            am_lfo: Phasor::default(),
            rm_lfo: Phasor::default(),
            current_freq: 330.0,
            pink_noise: PinkNoise::default(),
            brown_noise: BrownNoise::default(),
            #[cfg(not(feature = "native"))]
            file_source: None,
            #[cfg(feature = "native")]
            registry_sample: None,
            #[cfg(feature = "native")]
            registry_sample_b: None,
            sample_blend: 0.0,
            #[cfg(feature = "native")]
            stretch: StretchState::default(),
            web_sample: None,
            phaser: [Phaser::default(); CHANNELS],
            flanger: Some(Box::new([Flanger::default(); CHANNELS])),
            smear: [Smear::default(); CHANNELS],
            chorus: Some(Box::new(Chorus::default())),
            coarse: [Coarse::default(); CHANNELS],
            eq: [Eq::default(); CHANNELS],
            tilt: [Tilt::default(); CHANNELS],
            haas: Some(Box::new(Haas::default())),
            ladder_lp: [LadderFilter::default(); CHANNELS],
            ladder_hp: [LadderFilter::default(); CHANNELS],
            ladder_bp: [LadderFilter::default(); CHANNELS],
            param_mods: [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS],
            param_mod_count: 0,
            triggered: false,
            time: 0.0,
            ch: [0.0; CHANNELS],
            nch: 1,
            spread_side: 0.0,
            spread_cache_value: f32::NAN,
            spread_detune_ratios: [1.0; 3],
            additive_cache: AdditiveCache::default(),
            shape_active: false,
            sr,
            seed: 123456789,
            drum_svf: SvfState::default(),
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
            dahdsr: self.dahdsr,
            lp: self.lp,
            hp: self.hp,
            bp: self.bp,
            vib_lfo: self.vib_lfo,
            fm_phasor: self.fm_phasor,
            fm2_phasor: self.fm2_phasor,
            fm_fb_prev: self.fm_fb_prev,
            fm_fb_prev2: self.fm_fb_prev2,
            am_lfo: self.am_lfo,
            rm_lfo: self.rm_lfo,
            current_freq: self.current_freq,
            pink_noise: self.pink_noise,
            brown_noise: self.brown_noise,
            #[cfg(not(feature = "native"))]
            file_source: self.file_source,
            #[cfg(feature = "native")]
            registry_sample: self.registry_sample.clone(),
            #[cfg(feature = "native")]
            registry_sample_b: self.registry_sample_b.clone(),
            sample_blend: self.sample_blend,
            #[cfg(feature = "native")]
            stretch: self.stretch,
            web_sample: self.web_sample,
            phaser: self.phaser,
            flanger: self.flanger.clone(),
            smear: self.smear,
            chorus: self.chorus.clone(),
            coarse: self.coarse,
            eq: self.eq,
            tilt: self.tilt,
            haas: self.haas.clone(),
            ladder_lp: self.ladder_lp,
            ladder_hp: self.ladder_hp,
            ladder_bp: self.ladder_bp,
            param_mods: self.param_mods,
            param_mod_count: self.param_mod_count,
            triggered: self.triggered,
            time: self.time,
            ch: self.ch,
            nch: self.nch,
            spread_side: self.spread_side,
            spread_cache_value: self.spread_cache_value,
            spread_detune_ratios: self.spread_detune_ratios,
            additive_cache: self.additive_cache,
            shape_active: self.shape_active,
            sr: self.sr,
            seed: self.seed,
            drum_svf: self.drum_svf,
        }
    }
}

impl Voice {
    pub fn reset(&mut self) {
        self.params = VoiceParams::default();
        self.phasor = Phasor::default();
        self.sub_phasor = Phasor::default();
        for (i, p) in self.spread_phasors.iter_mut().enumerate() {
            *p = Phasor::default();
            p.phase = i as f32 / 7.0;
        }
        self.dahdsr = Dahdsr::default();
        self.lp = [SvfState::default(); CHANNELS];
        self.hp = [SvfState::default(); CHANNELS];
        self.bp = [SvfState::default(); CHANNELS];
        self.vib_lfo = Phasor::default();
        self.fm_phasor = Phasor::default();
        self.fm2_phasor = Phasor::default();
        self.fm_fb_prev = 0.0;
        self.fm_fb_prev2 = 0.0;
        self.am_lfo = Phasor::default();
        self.rm_lfo = Phasor::default();
        self.current_freq = 330.0;
        self.pink_noise = PinkNoise::default();
        self.brown_noise = BrownNoise::default();
        #[cfg(not(feature = "native"))]
        {
            self.file_source = None;
        }
        #[cfg(feature = "native")]
        {
            self.registry_sample = None;
            self.registry_sample_b = None;
        }
        self.sample_blend = 0.0;
        #[cfg(feature = "native")]
        {
            self.stretch = StretchState::default();
        }
        self.web_sample = None;
        self.phaser = [Phaser::default(); CHANNELS];
        if let Some(ref mut f) = self.flanger {
            **f = [Flanger::default(); CHANNELS];
        }
        self.smear = [Smear::default(); CHANNELS];
        if let Some(ref mut c) = self.chorus {
            **c = Chorus::default();
        }
        self.coarse = [Coarse::default(); CHANNELS];
        self.eq = [Eq::default(); CHANNELS];
        self.tilt = [Tilt::default(); CHANNELS];
        if let Some(ref mut h) = self.haas {
            **h = Haas::default();
        }
        self.ladder_lp = [LadderFilter::default(); CHANNELS];
        self.ladder_hp = [LadderFilter::default(); CHANNELS];
        self.ladder_bp = [LadderFilter::default(); CHANNELS];
        self.param_mods = [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS];
        self.param_mod_count = 0;
        self.triggered = false;
        self.time = 0.0;
        self.ch = [0.0; CHANNELS];
        self.nch = 1;
        self.spread_side = 0.0;
        self.spread_cache_value = f32::NAN;
        self.spread_detune_ratios = [1.0; 3];
        self.additive_cache = AdditiveCache::default();
        self.shape_active = false;
        self.sr = 44100.0;
        self.seed = 123456789;
        self.drum_svf = SvfState::default();
    }

    /// No-op: effects are pre-allocated at init.
    pub fn ensure_effects(&mut self) {}

    #[inline]
    pub(super) fn rand(&mut self) -> f32 {
        self.seed = modulation::lcg(self.seed);
        ((self.seed >> 16) & 0x7fff) as f32 / 32767.0
    }

    #[inline]
    pub(super) fn white(&mut self) -> f32 {
        self.rand() * 2.0 - 1.0
    }

    #[inline]
    pub(crate) fn spread_detune_ratios(&mut self) -> &[f32; 3] {
        if self.spread_cache_value != self.params.spread {
            for (i, ratio) in self.spread_detune_ratios.iter_mut().enumerate() {
                let detune_cents = ((i + 1) * (i + 1)) as f32 * self.params.spread;
                *ratio = exp2f(detune_cents / 1200.0);
            }
            self.spread_cache_value = self.params.spread;
        }
        &self.spread_detune_ratios
    }

    #[inline]
    pub(crate) fn sync_source_state(&mut self) {
        self.shape_active = self.params.shape.is_active();
        self.invalidate_additive_cache();
    }

    #[inline]
    fn invalidate_additive_cache(&mut self) {
        self.additive_cache.valid = false;
    }

    pub fn set_mod(&mut self, id: ParamId, chain: ModChain) {
        let chain = if let ModChain::Slew {
            target,
            freq,
            curve,
        } = chain
        {
            let start = self.read_param(id);
            ModChain::Transition {
                start,
                target,
                freq,
                curve,
                looping: false,
            }
        } else {
            chain
        };
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

    fn read_param(&self, id: ParamId) -> f32 {
        match id {
            ParamId::Freq => self.params.freq,
            ParamId::Gain => self.params.gain,
            ParamId::Postgain => self.params.postgain,
            ParamId::Pan => self.params.pan,
            ParamId::Speed => self.params.speed,
            ParamId::Stretch => self.params.stretch,
            ParamId::Detune => self.params.detune,
            ParamId::Pw => self.params.pw,
            ParamId::Wave => self.params.wave,
            ParamId::Sub => self.params.sub,
            ParamId::Harmonics => self.params.harmonics,
            ParamId::Timbre => self.params.timbre,
            ParamId::Morph => self.params.morph,
            ParamId::Scan => self.params.scan,
            ParamId::Mirror => self.params.shape.mirror,
            ParamId::Partials => self.params.partials,
            ParamId::Lpf => self.params.lpf.unwrap_or(20000.0),
            ParamId::Lpq => self.params.lpq,
            ParamId::Hpf => self.params.hpf.unwrap_or(0.0),
            ParamId::Hpq => self.params.hpq,
            ParamId::Bpf => self.params.bpf.unwrap_or(1000.0),
            ParamId::Bpq => self.params.bpq,
            ParamId::Llpf => self.params.llpf.unwrap_or(20000.0),
            ParamId::Llpq => self.params.llpq,
            ParamId::Lhpf => self.params.lhpf.unwrap_or(0.0),
            ParamId::Lhpq => self.params.lhpq,
            ParamId::Lbpf => self.params.lbpf.unwrap_or(1000.0),
            ParamId::Lbpq => self.params.lbpq,
            ParamId::Fm => self.params.fm,
            ParamId::Fmh => self.params.fmh,
            ParamId::Fm2 => self.params.fm2,
            ParamId::Fm2h => self.params.fm2h,
            ParamId::Fmfb => self.params.fmfb,
            ParamId::Am => self.params.am,
            ParamId::Amdepth => self.params.amdepth,
            ParamId::Rm => self.params.rm,
            ParamId::Rmdepth => self.params.rmdepth,
            ParamId::Vib => self.params.vib,
            ParamId::Vibmod => self.params.vibmod,
            ParamId::Phaser => self.params.phaser,
            ParamId::Phaserdepth => self.params.phaserdepth,
            ParamId::Phasersweep => self.params.phasersweep,
            ParamId::Phasercenter => self.params.phasercenter,
            ParamId::Flanger => self.params.flanger,
            ParamId::Flangerdepth => self.params.flangerdepth,
            ParamId::Flangerfeedback => self.params.flangerfeedback,
            ParamId::Smear => self.params.smear,
            ParamId::Smearfreq => self.params.smearfreq,
            ParamId::Smearfb => self.params.smearfb,
            ParamId::Chorus => self.params.chorus,
            ParamId::Chorusdepth => self.params.chorusdepth,
            ParamId::Chorusdelay => self.params.chorusdelay,
            ParamId::Fold => self.params.fold.unwrap_or(0.0),
            ParamId::Crush => self.params.crush.unwrap_or(0.0),
            ParamId::Coarse => self.params.coarse.unwrap_or(0.0),
            ParamId::Distort => self.params.distort.unwrap_or(0.0),
            ParamId::Wrap => self.params.wrap.unwrap_or(0.0),
            ParamId::Eqlo => self.params.eqlo,
            ParamId::Eqmid => self.params.eqmid,
            ParamId::Eqhi => self.params.eqhi,
            ParamId::Tilt => self.params.tilt,
            ParamId::Width => self.params.width,
            ParamId::Haas => self.params.haas,
            ParamId::Delay => self.params.delay,
            ParamId::Verb => self.params.verb,
            ParamId::Comb => self.params.comb,
            ParamId::Feedback => self.params.feedback,
            ParamId::FbTime => self.params.fbtime,
            ParamId::CombFreq => self.params.combfreq,
            ParamId::CombFeedback => self.params.combfeedback,
            ParamId::DelayTime => self.params.delaytime,
            ParamId::DelayFeedback => self.params.delayfeedback,
            ParamId::EqLoFreq => self.params.eqlofreq,
            ParamId::EqMidFreq => self.params.eqmidfreq,
            ParamId::EqHiFreq => self.params.eqhifreq,
            ParamId::Comp => self.params.comp,
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
            ParamId::Stretch => self.params.stretch = val,
            ParamId::Detune => self.params.detune = val,
            ParamId::Pw => self.params.pw = val,
            ParamId::Wave => self.params.wave = val,
            ParamId::Sub => self.params.sub = val,
            ParamId::Harmonics => {
                self.params.harmonics = val;
                self.invalidate_additive_cache();
            }
            ParamId::Timbre => {
                self.params.timbre = val;
                self.invalidate_additive_cache();
            }
            ParamId::Morph => {
                self.params.morph = val;
                self.invalidate_additive_cache();
            }
            ParamId::Scan => self.params.scan = val,
            ParamId::Mirror => {
                self.params.shape.mirror = val;
                self.shape_active = self.params.shape.is_active();
            }
            ParamId::Partials => {
                self.params.partials = val;
                self.invalidate_additive_cache();
            }
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
            ParamId::Smear => self.params.smear = val,
            ParamId::Smearfreq => self.params.smearfreq = val,
            ParamId::Smearfb => self.params.smearfb = val,
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
            ParamId::FbTime => self.params.fbtime = val,
            ParamId::CombFreq => self.params.combfreq = val,
            ParamId::CombFeedback => self.params.combfeedback = val,
            ParamId::DelayTime => self.params.delaytime = val,
            ParamId::DelayFeedback => self.params.delayfeedback = val,
            ParamId::EqLoFreq => self.params.eqlofreq = val,
            ParamId::EqMidFreq => self.params.eqmidfreq = val,
            ParamId::EqHiFreq => self.params.eqhifreq = val,
            ParamId::Comp => self.params.comp = val,
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

        // FM synthesis (3-operator)
        if self.params.fm > 0.0 || self.params.fm2 > 0.0 {
            let fm1 = self.params.fm;
            let fm2 = self.params.fm2;
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

        // Vibrato
        if self.params.vib > 0.0 && self.params.vibmod > 0.0 {
            let mod_val = self.vib_lfo.lfo(self.params.vibshape, self.params.vib, isr);
            freq *= exp2f(mod_val * self.params.vibmod / 12.0);
        }

        self.current_freq = freq;
        freq
    }

    pub fn force_release(&mut self) {
        self.dahdsr.force_release();
        for i in 0..self.param_mod_count as usize {
            self.param_mods[i].1.force_release();
        }
    }

    /// Cut this voice immediately (~1ms fade to avoid clicks).
    pub fn hard_cut(&mut self) {
        self.params.release = 0.001;
        self.force_release();
    }

    fn trigger_envelopes(&mut self) {
        self.dahdsr.trigger(self.params.gate);
        for i in 0..self.param_mod_count as usize {
            self.param_mods[i].1.trigger(self.params.gate);
        }
    }

    pub(crate) fn prepare_frame(&mut self, isr: f32) -> Option<(f32, f32)> {
        if !self.triggered {
            self.trigger_envelopes();
            self.triggered = true;
        }

        let env = self.dahdsr.update(
            isr,
            self.params.envdelay,
            self.params.attack,
            self.params.hold,
            self.params.decay,
            self.params.sustain,
            self.params.release,
        );
        if self.dahdsr.is_off() {
            return None;
        }

        if self.param_mod_count > 0 {
            self.apply_mods(isr);
        }

        Some((env, self.compute_freq(isr)))
    }

    #[cfg(feature = "native")]
    pub fn process(
        &mut self,
        isr: f32,
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
        input_channels: usize,
    ) -> bool {
        let Some((env, freq)) = self.prepare_frame(isr) else {
            return false;
        };

        if !self.run_source(freq, isr, web_pcm, sample_idx, live_input, input_channels) {
            return false;
        }
        self.apply_filters_and_effects(env, isr);
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
        input_channels: usize,
    ) -> bool {
        let Some((env, freq)) = self.prepare_frame(isr) else {
            return false;
        };

        if !self.run_source(
            freq,
            isr,
            pool,
            samples,
            web_pcm,
            sample_idx,
            live_input,
            input_channels,
        ) {
            return false;
        }

        self.apply_filters_and_effects(env, isr);
        true
    }

    #[inline]
    pub(crate) fn apply_filters_and_effects(&mut self, env: f32, isr: f32) {
        let nch = self.nch;

        // Update filter cutoffs
        if let Some(lpf) = self.params.lpf {
            for c in 0..nch {
                self.lp[c].cutoff = lpf;
            }
        }
        if let Some(hpf) = self.params.hpf {
            for c in 0..nch {
                self.hp[c].cutoff = hpf;
            }
        }
        if let Some(bpf) = self.params.bpf {
            for c in 0..nch {
                self.bp[c].cutoff = bpf;
            }
        }

        // Pre-filter gain
        for c in 0..nch {
            self.ch[c] *= self.params.gain * self.params.velocity;
        }

        // SVF filters (LP -> HP -> BP)
        if self.params.lpf.is_some() {
            for c in 0..nch {
                self.ch[c] = self.lp[c].process(self.ch[c], SvfMode::Lp, self.params.lpq, self.sr);
            }
        }
        if self.params.hpf.is_some() {
            for c in 0..nch {
                self.ch[c] = self.hp[c].process(self.ch[c], SvfMode::Hp, self.params.hpq, self.sr);
            }
        }
        if self.params.bpf.is_some() {
            for c in 0..nch {
                self.ch[c] = self.bp[c].process(self.ch[c], SvfMode::Bp, self.params.bpq, self.sr);
            }
        }

        // Ladder filters
        if let Some(llpf) = self.params.llpf {
            for c in 0..nch {
                self.ch[c] = self.ladder_lp[c].process(
                    self.ch[c],
                    llpf,
                    self.params.llpq,
                    LadderMode::Lp,
                    self.sr,
                );
            }
        }
        if let Some(lhpf) = self.params.lhpf {
            for c in 0..nch {
                self.ch[c] = self.ladder_hp[c].process(
                    self.ch[c],
                    lhpf,
                    self.params.lhpq,
                    LadderMode::Hp,
                    self.sr,
                );
            }
        }
        if let Some(lbpf) = self.params.lbpf {
            for c in 0..nch {
                self.ch[c] = self.ladder_bp[c].process(
                    self.ch[c],
                    lbpf,
                    self.params.lbpq,
                    LadderMode::Bp,
                    self.sr,
                );
            }
        }

        // Distortion effects
        if let Some(coarse_factor) = self.params.coarse {
            for c in 0..nch {
                self.ch[c] = self.coarse[c].process(self.ch[c], coarse_factor);
            }
        }
        if let Some(crush_bits) = self.params.crush {
            for c in 0..nch {
                self.ch[c] = crush(self.ch[c], crush_bits);
            }
        }
        if let Some(fold_amount) = self.params.fold {
            for c in 0..nch {
                self.ch[c] = fold(self.ch[c], fold_amount);
            }
        }
        if let Some(wrap_amount) = self.params.wrap {
            for c in 0..nch {
                self.ch[c] = wrap(self.ch[c], wrap_amount);
            }
        }
        if let Some(dist_amount) = self.params.distort {
            for c in 0..nch {
                self.ch[c] = distort(self.ch[c], dist_amount, self.params.distortvol);
            }
        }

        // AM modulation (LFO ticks once, applied per-channel)
        if self.params.am > 0.0 {
            let modulator = self.am_lfo.lfo(self.params.amshape, self.params.am, isr);
            let depth = self.params.amdepth.clamp(0.0, 1.0);
            let factor = 1.0 + modulator * depth;
            for c in 0..nch {
                self.ch[c] *= factor;
            }
        }

        // Ring modulation
        if self.params.rm > 0.0 {
            let modulator = self.rm_lfo.lfo(self.params.rmshape, self.params.rm, isr);
            let depth = self.params.rmdepth.clamp(0.0, 1.0);
            let factor = (1.0 - depth) + modulator * depth;
            for c in 0..nch {
                self.ch[c] *= factor;
            }
        }

        // Phaser
        if self.params.phaser > 0.0 {
            for c in 0..nch {
                self.ch[c] = self.phaser[c].process(
                    self.ch[c],
                    self.params.phaser,
                    self.params.phaserdepth,
                    self.params.phasercenter,
                    self.params.phasersweep,
                    self.sr,
                    isr,
                );
            }
        }

        // Flanger (must be pre-allocated via ensure_effects)
        if self.params.flanger > 0.0 {
            if let Some(flanger) = self.flanger.as_mut() {
                for c in 0..nch {
                    self.ch[c] = flanger[c].process(
                        self.ch[c],
                        self.params.flanger,
                        self.params.flangerdepth,
                        self.params.flangerfeedback,
                        self.sr,
                        isr,
                    );
                }
            }
        }

        // EQ
        if self.params.eqlo != 0.0 || self.params.eqmid != 0.0 || self.params.eqhi != 0.0 {
            for c in 0..nch {
                self.ch[c] = self.eq[c].process(
                    self.ch[c],
                    self.params.eqlo,
                    self.params.eqmid,
                    self.params.eqhi,
                    self.params.eqlofreq,
                    self.params.eqmidfreq,
                    self.params.eqhifreq,
                    self.sr,
                );
            }
        }

        // Tilt
        if self.params.tilt != 0.0 {
            for c in 0..nch {
                self.ch[c] = self.tilt[c].process(self.ch[c], self.params.tilt, self.sr);
            }
        }

        // Smear
        if self.params.smear > 0.0 {
            for c in 0..nch {
                self.ch[c] = self.smear[c].process(
                    self.ch[c],
                    self.params.smear,
                    self.params.smearfreq,
                    self.params.smearfb,
                    self.sr,
                );
            }
        }

        // Apply gain envelope and postgain
        for c in 0..nch {
            self.ch[c] *= env * self.params.postgain;
        }

        // Mono sources: spread or duplicate to stereo
        if nch == 1 {
            if self.params.spread > 0.0 {
                let side = self.spread_side * env * self.params.postgain;
                self.ch[1] = self.ch[0] - side;
                self.ch[0] += side;
            } else {
                self.ch[1] = self.ch[0];
            }
        }

        // Chorus (must be pre-allocated via ensure_effects)
        if self.params.chorus > 0.0 {
            if let Some(chorus) = self.chorus.as_mut() {
                let stereo = chorus.process(
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
        }

        // Stereo width (mid-side matrix)
        if self.params.width != 1.0 {
            let mid = (self.ch[0] + self.ch[1]) * 0.5;
            let side = (self.ch[0] - self.ch[1]) * 0.5;
            let w = self.params.width.max(0.0);
            self.ch[0] = mid + side * w;
            self.ch[1] = mid - side * w;
        }

        // Haas (must be pre-allocated via ensure_effects)
        if self.params.haas > 0.0 {
            if let Some(haas) = self.haas.as_mut() {
                self.ch[1] = haas.process(self.ch[1], self.params.haas, self.sr);
            }
        }

        // Panning
        if self.params.pan != 0.5 {
            let pan_pos = self.params.pan * PI / 2.0;
            self.ch[0] *= cosf(pan_pos);
            self.ch[1] *= sinf(pan_pos);
        }

        self.time += isr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_cache_invalidates_on_reset() {
        let mut voice = Voice::default();
        voice.params.timbre = 0.7;
        voice.additive_cache.valid = true;
        voice.shape_active = true;

        voice.reset();

        assert!(!voice.additive_cache.valid);
        assert!(!voice.shape_active);
    }

    #[test]
    fn additive_cache_invalidates_for_additive_params_only() {
        let mut voice = Voice::default();
        voice.additive_cache.valid = true;

        voice.write_param(ParamId::Gain, 0.8);
        assert!(voice.additive_cache.valid);

        voice.write_param(ParamId::Timbre, 0.7);
        assert!(!voice.additive_cache.valid);

        voice.additive_cache.valid = true;
        voice.write_param(ParamId::Partials, 12.0);
        assert!(!voice.additive_cache.valid);
    }

    #[test]
    fn sync_source_state_refreshes_shape_activity() {
        let mut voice = Voice::default();
        voice.params.shape.size = 8;
        voice.sync_source_state();
        assert!(voice.shape_active);

        voice.params.shape = crate::dsp::PhaseShape::default();
        voice.sync_source_state();
        assert!(!voice.shape_active);
    }
}
