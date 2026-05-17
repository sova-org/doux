//! Voice - the core synthesis unit.

mod drums;
pub mod modulation;
mod params;
mod source;

pub use modulation::{ModChain, ParamId, ParamMod};
pub use params::VoiceParams;

use std::f32::consts::PI;

use crate::dsp::{
    cosf, exp2f, sinf, BrownNoise, Dahdsr, Phasor, PinkNoise, SvfCascade, SvfMode, SvfState,
};
use crate::effects::{
    Chorus, Coarse, DcBlocker, Eq, Flanger, Fold, Haas, LadderFilter, LadderMode, Phaser, Smear,
    Tilt, Wrap,
};
#[cfg(feature = "native")]
use crate::sampling::RegistrySample;
#[cfg(feature = "native")]
use crate::sampling::StretchState;
use crate::sampling::WebSampleSource;
#[cfg(not(feature = "native"))]
use crate::sampling::{FileSource, SampleInfo};
use crate::types::{StereoFrame, CHANNELS, MAX_BLOCK};

pub const MAX_PARAM_MODS: usize = 15;
pub(crate) const MAX_ADDITIVE_PARTIALS: usize = 32;
const VOICE_OUTPUT_TRIM: f32 = 0.5;
/// `1 / (2π)`: converts radians to turns for phase-modulation math.
const INV_TAU: f32 = 0.159_154_94;

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

/// Layout: hot fields (touched every block, every active voice) cluster at the
/// top; cold FX state (24 dB/oct cascades, ladder, distortion stages, EQ/tilt,
/// phaser/flanger/smear/chorus/haas) sits in the cold tail since most voices
/// run with the FX gates disabled. Reordering recovers cache locality during
/// the per-voice block kernel: ≈ 5 KiB working set per voice-block including
/// scratch and orbit bus, fits in the smallest L1d in the target class.
#[derive(Clone)]
pub struct Voice {
    // === Hot: source generation + always-active per-block state ===
    pub params: VoiceParams,
    /// Per-voice block scratch. Source code writes directly into
    /// `scratch[i][c]`; FX block APIs operate on this in place. Allocated once
    /// at construction; cleared via `*self.scratch = [[0.0; CHANNELS]; MAX_BLOCK]`
    /// in `reset()` (no realloc).
    pub scratch: Box<[StereoFrame; MAX_BLOCK]>,
    pub dahdsr: Dahdsr,
    pub phasor: Phasor,
    pub sub_phasor: Phasor,
    pub sync_phasor: Phasor,
    /// Soft-sync direction: `+1.0` forward, `-1.0` reversed. Flips on master wrap in `Soft` mode.
    pub sync_direction: f32,
    pub spread_phasors: [Phasor; 7],
    pub vib_lfo: Phasor,
    pub fm_phasor: Phasor,
    pub fm2_phasor: Phasor,
    pub fm_fb_prev: f32,
    pub fm_fb_prev2: f32,
    /// Phase-modulation offset applied to the carrier read, in turns.
    /// Computed once per sample by `compute_freq`; read by `generate_main_osc`.
    pub fm_phase_mod: f32,
    pub am_lfo: Phasor,
    pub rm_lfo: Phasor,
    pub current_freq: f32,
    pub lp: [SvfState; CHANNELS],
    pub hp: [SvfState; CHANNELS],
    pub bp: [SvfState; CHANNELS],
    pub nch: usize,
    pub spread_side: f32,
    pub spread_cache_value: f32,
    pub spread_detune_ratios: [f32; 3],
    pub triggered: bool,
    pub time: f32,
    pub sr: f32,
    pub seed: u32,
    pub(crate) additive_cache: AdditiveCache,
    pub(crate) shape_active: bool,

    // === Source-specific state (one variant active per voice) ===
    pub pink_noise: PinkNoise,
    pub brown_noise: BrownNoise,
    #[cfg(not(feature = "native"))]
    pub file_source: Option<FileSource>,
    #[cfg(feature = "native")]
    pub registry_sample: Option<RegistrySample>,
    #[cfg(feature = "native")]
    pub registry_sample_b: Option<RegistrySample>,
    pub sample_blend: f32,
    #[cfg(feature = "native")]
    pub stretch: StretchState,
    pub web_sample: Option<WebSampleSource>,
    pub(super) drum_svf: SvfState,

    // === Param modulation (read once per block in `apply_mods`) ===
    pub param_mods: [(ParamId, ParamMod); MAX_PARAM_MODS],
    pub param_mod_count: u8,

    // === Cold: FX state. Conditional per stage; cold for voices with the
    // gate disabled. Steep SVF cascades alone are ≈ 504 B; clustering at the
    // tail keeps the hot working set inside L1d.
    pub slp: [SvfCascade; CHANNELS],
    pub shp: [SvfCascade; CHANNELS],
    pub sbp: [SvfCascade; CHANNELS],
    pub ladder_lp: [LadderFilter; CHANNELS],
    pub ladder_hp: [LadderFilter; CHANNELS],
    pub ladder_bp: [LadderFilter; CHANNELS],
    pub coarse: [Coarse; CHANNELS],
    pub fold_state: [Fold; CHANNELS],
    pub wrap_state: [Wrap; CHANNELS],
    pub dc_block: [DcBlocker; CHANNELS],
    pub eq: [Eq; CHANNELS],
    pub tilt: [Tilt; CHANNELS],
    pub phaser: [Phaser; CHANNELS],
    pub flanger: Option<Box<[Flanger; CHANNELS]>>,
    pub smear: [Smear; CHANNELS],
    pub chorus: Option<Box<Chorus>>,
    pub haas: Option<Box<Haas>>,
}

impl Default for Voice {
    fn default() -> Self {
        let sr = 44100.0;
        Self {
            params: VoiceParams::default(),
            scratch: Box::new([[0.0; CHANNELS]; MAX_BLOCK]),
            dahdsr: Dahdsr::default(),
            phasor: Phasor::default(),
            sub_phasor: Phasor::default(),
            sync_phasor: Phasor::default(),
            sync_direction: 1.0,
            spread_phasors: std::array::from_fn(|i| {
                let mut p = Phasor::default();
                p.phase = i as f32 / 7.0;
                p
            }),
            vib_lfo: Phasor::default(),
            fm_phasor: Phasor::default(),
            fm2_phasor: Phasor::default(),
            fm_fb_prev: 0.0,
            fm_fb_prev2: 0.0,
            fm_phase_mod: 0.0,
            am_lfo: Phasor::default(),
            rm_lfo: Phasor::default(),
            current_freq: 330.0,
            lp: [SvfState::default(); CHANNELS],
            hp: [SvfState::default(); CHANNELS],
            bp: [SvfState::default(); CHANNELS],
            nch: 1,
            spread_side: 0.0,
            spread_cache_value: f32::NAN,
            spread_detune_ratios: [1.0; 3],
            triggered: false,
            time: 0.0,
            sr,
            seed: 123456789,
            additive_cache: AdditiveCache::default(),
            shape_active: false,
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
            drum_svf: SvfState::default(),
            param_mods: [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS],
            param_mod_count: 0,
            slp: [SvfCascade::default(); CHANNELS],
            shp: [SvfCascade::default(); CHANNELS],
            sbp: [SvfCascade::default(); CHANNELS],
            ladder_lp: [LadderFilter::default(); CHANNELS],
            ladder_hp: [LadderFilter::default(); CHANNELS],
            ladder_bp: [LadderFilter::default(); CHANNELS],
            coarse: [Coarse::default(); CHANNELS],
            fold_state: [Fold::default(); CHANNELS],
            wrap_state: [Wrap::default(); CHANNELS],
            dc_block: [DcBlocker::default(); CHANNELS],
            eq: [Eq::default(); CHANNELS],
            tilt: [Tilt::default(); CHANNELS],
            phaser: [Phaser::default(); CHANNELS],
            flanger: Some(Box::new([Flanger::default(); CHANNELS])),
            smear: [Smear::default(); CHANNELS],
            chorus: Some(Box::new(Chorus::default())),
            haas: Some(Box::new(Haas::default())),
        }
    }
}

impl Voice {
    pub fn reset(&mut self) {
        self.params = VoiceParams::default();
        self.phasor = Phasor::default();
        self.sub_phasor = Phasor::default();
        self.sync_phasor = Phasor::default();
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
        self.fm_phase_mod = 0.0;
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
        self.fold_state = [Fold::default(); CHANNELS];
        self.wrap_state = [Wrap::default(); CHANNELS];
        self.dc_block = [DcBlocker::default(); CHANNELS];
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
        *self.scratch = [[0.0; CHANNELS]; MAX_BLOCK];
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
            ParamId::SyncRatio => self.params.sync_ratio,
            ParamId::SyncPhase => self.params.sync_phase,
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
            ParamId::Slpf => self.params.slpf.unwrap_or(20000.0),
            ParamId::Slpq => self.params.slpq,
            ParamId::Shpf => self.params.shpf.unwrap_or(0.0),
            ParamId::Shpq => self.params.shpq,
            ParamId::Sbpf => self.params.sbpf.unwrap_or(1000.0),
            ParamId::Sbpq => self.params.sbpq,
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
            ParamId::Fmpivot => self.params.fmpivot,
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
            ParamId::EqLoFreq => self.params.eqlofreq,
            ParamId::EqMidFreq => self.params.eqmidfreq,
            ParamId::EqHiFreq => self.params.eqhifreq,
        }
    }

    /// Block-rate param-mod application. Advances each `ParamMod` by `n` per-sample
    /// steps internally and writes the post-block value to its target once.
    #[inline]
    fn apply_mods_block(&mut self, isr: f32, n: usize) {
        for i in 0..self.param_mod_count as usize {
            let (id, ref mut m) = self.param_mods[i];
            let val = m.tick_block(isr, n);
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
            ParamId::SyncRatio => self.params.sync_ratio = val,
            ParamId::SyncPhase => self.params.sync_phase = val,
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
            ParamId::Slpf => self.params.slpf = Some(val),
            ParamId::Slpq => self.params.slpq = val,
            ParamId::Shpf => self.params.shpf = Some(val),
            ParamId::Shpq => self.params.shpq = val,
            ParamId::Sbpf => self.params.sbpf = Some(val),
            ParamId::Sbpq => self.params.sbpq = val,
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
            ParamId::Fmpivot => self.params.fmpivot = val,
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
            ParamId::EqLoFreq => self.params.eqlofreq = val,
            ParamId::EqMidFreq => self.params.eqmidfreq = val,
            ParamId::EqHiFreq => self.params.eqhifreq = val,
        }
    }

    /// Block-rate carrier frequency: applies `detune`, `speed`, and `vib` once
    /// per block. Advances `vib_lfo` by one tick. Stores post-vib freq in
    /// `self.current_freq` and returns it.
    ///
    /// FM phase modulation is **not** computed here; it runs per-sample inside
    /// [`Voice::run_source_block`] (see `tick_fm_pm`). The pre-vib carrier freq
    /// used by FM modulators is recomputed there cheaply (detune + speed; same
    /// block-rate scalars).
    fn compute_freq_block(&mut self, isr: f32) -> f32 {
        let mut freq = self.params.freq;
        if self.params.detune != 0.0 {
            freq *= exp2f(self.params.detune / 1200.0);
        }
        freq *= self.params.speed;
        if self.params.vib > 0.0 && self.params.vibmod > 0.0 {
            let mod_val = self.vib_lfo.lfo(self.params.vibshape, self.params.vib, isr);
            freq *= exp2f(mod_val * self.params.vibmod / 12.0);
        }
        self.current_freq = freq;
        freq
    }

    /// Pre-vib carrier frequency for FM modulators. Cheap: detune + speed only.
    /// Block-rate inputs (`self.params.detune`, `self.params.speed`) are stable
    /// for the whole block after `apply_mods_block` runs, so recomputing here
    /// avoids stashing the value across calls.
    #[inline]
    pub(crate) fn fm_carrier_freq(&self) -> f32 {
        let mut f = self.params.freq;
        if self.params.detune != 0.0 {
            f *= exp2f(self.params.detune / 1200.0);
        }
        f *= self.params.speed;
        f
    }

    /// Per-sample FM phase-modulation tick. Writes `self.fm_phase_mod` and
    /// advances `fm_phasor` / `fm2_phasor` by one sample. `freq_pre_vib` is
    /// the carrier freq before vibrato (FM modulators tick at this freq to
    /// preserve original sample-rate ordering: detune → speed → FM → vib).
    ///
    /// `fmpivot` rotates op2's output through a circle in the
    /// (op2→op1, op2→carrier) plane: 0 = cascade, 0.125 = branch,
    /// 0.25 = parallel, 0.5 = inverted cascade, etc. Total op2 modulation
    /// magnitude `√(a²+b²) = fm2` is constant; only the destination rotates.
    #[inline]
    pub(crate) fn tick_fm_pm(&mut self, freq_pre_vib: f32, isr: f32) {
        let mut pm = 0.0_f32;
        if self.params.fm > 0.0 || self.params.fm2 > 0.0 {
            let fm1 = self.params.fm;
            let fm2 = self.params.fm2;
            let shape = self.params.fmshape;
            let fb_turns = (self.fm_fb_prev + self.fm_fb_prev2) * 0.5 * self.params.fmfb * INV_TAU;

            if fm2 > 0.0 {
                let theta = self.params.fmpivot * std::f32::consts::TAU;
                let a = fm2 * cosf(theta); // op2 → op1
                let b = fm2 * sinf(theta); // op2 → carrier

                let mod2_freq = freq_pre_vib * self.params.fm2h;
                let mod2 = self.fm2_phasor.lfo_pm(shape, mod2_freq, isr, fb_turns);
                self.fm_fb_prev2 = self.fm_fb_prev;
                self.fm_fb_prev = mod2;

                let mod1_freq = freq_pre_vib * self.params.fmh;
                let mod1 = self
                    .fm_phasor
                    .lfo_pm(shape, mod1_freq, isr, a * mod2 * INV_TAU);

                pm += fm1 * mod1 * INV_TAU;
                pm += b * mod2 * INV_TAU;
            } else {
                let mod1_freq = freq_pre_vib * self.params.fmh;
                let mod1 = self.fm_phasor.lfo_pm(shape, mod1_freq, isr, fb_turns);
                self.fm_fb_prev2 = self.fm_fb_prev;
                self.fm_fb_prev = mod1;
                pm += fm1 * mod1 * INV_TAU;
            }
        }
        self.fm_phase_mod = pm;
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
        self.sync_direction = 1.0;
        for i in 0..self.param_mod_count as usize {
            self.param_mods[i].1.trigger(self.params.gate);
        }
    }

    /// Block-rate preamble: trigger the envelope if needed, advance it `n`
    /// samples, run param-mods (block-rate), and compute the block-rate carrier
    /// frequency. Returns the stack-allocated envelope buffer and the post-vib
    /// carrier freq, or `None` if the envelope is `Off` after the block.
    ///
    /// **Bit-identity at `n = 1`**: matches the legacy `prepare_frame` ordering
    /// — `update` first, `is_off()` check second, then `apply_mods` and
    /// `compute_freq`. The post-update `is_off()` check is load-bearing: if
    /// the envelope transitions to `Off` during this sample, the voice is
    /// considered dead and the caller frees it without producing output.
    pub(crate) fn prepare_block(&mut self, isr: f32, n: usize) -> Option<([f32; MAX_BLOCK], f32)> {
        if !self.triggered {
            self.trigger_envelopes();
            self.triggered = true;
        }

        let mut env: [f32; MAX_BLOCK] = [0.0; MAX_BLOCK];
        self.dahdsr.update_block(
            &mut env,
            n,
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
            self.apply_mods_block(isr, n);
        }

        Some((env, self.compute_freq_block(isr)))
    }

    /// Orchestrates block-internal voice processing. Returns the number of
    /// samples written to `self.scratch[..n]`. Samples beyond `written` are
    /// zeroed so the caller can mix `self.scratch[..n]` unconditionally.
    ///
    /// Layout: `prepare_block` → `run_source_block` → `apply_filters_and_effects_block`.
    /// `#[inline(never)]` because the function is large; inlining at every
    /// call site would blow the caller's I-cache.
    #[inline(never)]
    #[cfg(feature = "native")]
    pub fn process_block(
        &mut self,
        n: usize,
        isr: f32,
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
        input_channels: usize,
    ) -> usize {
        debug_assert!(
            n <= MAX_BLOCK,
            "Voice::process_block: n={n} > MAX_BLOCK={MAX_BLOCK}"
        );
        let Some((env, freq)) = self.prepare_block(isr, n) else {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        };

        let written = self.run_source_block(
            freq,
            isr,
            n,
            web_pcm,
            sample_idx,
            live_input,
            input_channels,
        );
        if written == 0 {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        }
        for i in written..n {
            self.scratch[i] = [0.0; CHANNELS];
        }

        self.apply_filters_and_effects_block(&env, isr, written);
        written
    }

    #[inline(never)]
    #[cfg(not(feature = "native"))]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        n: usize,
        isr: f32,
        pool: &[f32],
        samples: &[SampleInfo],
        web_pcm: &[f32],
        sample_idx: usize,
        live_input: &[f32],
        input_channels: usize,
    ) -> usize {
        debug_assert!(
            n <= MAX_BLOCK,
            "Voice::process_block: n={n} > MAX_BLOCK={MAX_BLOCK}"
        );
        let Some((env, freq)) = self.prepare_block(isr, n) else {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        };

        let written = self.run_source_block(
            freq,
            isr,
            n,
            pool,
            samples,
            web_pcm,
            sample_idx,
            live_input,
            input_channels,
        );
        if written == 0 {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        }
        for i in written..n {
            self.scratch[i] = [0.0; CHANNELS];
        }

        self.apply_filters_and_effects_block(&env, isr, written);
        written
    }

    /// Block-rate FX chain. **Option/branch hoist**: every `if let Some(...)` /
    /// `if self.params.X > 0.0` gate evaluates **once** at block entry; each
    /// enabled FX stage then runs its own `for i in 0..n` inner loop over
    /// `self.scratch[..n]`. The codegen check at the end of Phase D confirms
    /// the inner loops are free of `Option`-discriminant tests.
    ///
    /// Param-rate boundary (see `to_do.md:158-167`):
    /// - Block-rate (read from `self.params` once at block entry): all filter
    ///   cutoffs, distortion threshold/depth, EQ gains, gain/postgain/pan,
    ///   width, haas, phaser/flanger/smear/chorus rates.
    /// - Sample-rate (advanced n times inside the kernel): AM/RM modulator
    ///   LFOs, every DSP feedback path (filters, ladder, phaser, flanger,
    ///   smear, chorus, haas) — preserved by the per-FX `process_block` calls.
    #[allow(clippy::needless_range_loop)]
    pub(crate) fn apply_filters_and_effects_block(&mut self, env_buf: &[f32], isr: f32, n: usize) {
        let nch = self.nch;
        let sr = self.sr;

        // Pre-filter gain.
        let gain = self.params.gain;
        for frame in self.scratch[..n].iter_mut() {
            for c in 0..nch {
                frame[c] *= gain;
            }
        }

        // SVF filters (LP → HP → BP). Block-rate cutoff write; coefficient
        // recompute hoists inside `SvfState::process_block`.
        if let Some(lpf) = self.params.lpf {
            let q = self.params.lpq;
            for c in 0..nch {
                self.lp[c].cutoff = lpf;
                self.lp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Lp, q, sr);
            }
        }
        if let Some(hpf) = self.params.hpf {
            let q = self.params.hpq;
            for c in 0..nch {
                self.hp[c].cutoff = hpf;
                self.hp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Hp, q, sr);
            }
        }
        if let Some(bpf) = self.params.bpf {
            let q = self.params.bpq;
            for c in 0..nch {
                self.bp[c].cutoff = bpf;
                self.bp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Bp, q, sr);
            }
        }

        // Steep SVF cascades (24 dB/oct).
        if let Some(slpf) = self.params.slpf {
            let q = self.params.slpq;
            for c in 0..nch {
                self.slp[c].cutoff = slpf;
                self.slp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Lp, q, sr);
            }
        }
        if let Some(shpf) = self.params.shpf {
            let q = self.params.shpq;
            for c in 0..nch {
                self.shp[c].cutoff = shpf;
                self.shp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Hp, q, sr);
            }
        }
        if let Some(sbpf) = self.params.sbpf {
            let q = self.params.sbpq;
            for c in 0..nch {
                self.sbp[c].cutoff = sbpf;
                self.sbp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Bp, q, sr);
            }
        }

        // Ladder filters.
        if let Some(llpf) = self.params.llpf {
            let q = self.params.llpq;
            for c in 0..nch {
                self.ladder_lp[c].process_block(
                    &mut self.scratch[..n],
                    n,
                    c,
                    llpf,
                    q,
                    LadderMode::Lp,
                    sr,
                );
            }
        }
        if let Some(lhpf) = self.params.lhpf {
            let q = self.params.lhpq;
            for c in 0..nch {
                self.ladder_hp[c].process_block(
                    &mut self.scratch[..n],
                    n,
                    c,
                    lhpf,
                    q,
                    LadderMode::Hp,
                    sr,
                );
            }
        }
        if let Some(lbpf) = self.params.lbpf {
            let q = self.params.lbpq;
            for c in 0..nch {
                self.ladder_bp[c].process_block(
                    &mut self.scratch[..n],
                    n,
                    c,
                    lbpf,
                    q,
                    LadderMode::Bp,
                    sr,
                );
            }
        }

        // Distortion effects.
        if let Some(coarse_factor) = self.params.coarse {
            for c in 0..nch {
                self.coarse[c].process_block(&mut self.scratch[..n], n, c, coarse_factor);
            }
        }
        if let Some(crush_bits) = self.params.crush {
            let bits = crush_bits.max(1.0);
            let x = exp2f(bits - 1.0);
            let inv_x = 1.0 / x;
            for frame in self.scratch[..n].iter_mut() {
                for c in 0..nch {
                    frame[c] = (frame[c] * x).round() * inv_x;
                }
            }
        }
        if let Some(fold_amount) = self.params.fold {
            for c in 0..nch {
                self.fold_state[c].process_block(&mut self.scratch[..n], n, c, fold_amount);
            }
        }
        if let Some(wrap_amount) = self.params.wrap {
            for c in 0..nch {
                self.wrap_state[c].process_block(&mut self.scratch[..n], n, c, wrap_amount);
            }
        }
        if let Some(dist_amount) = self.params.distort {
            let postgain = self.params.distortvol;
            let k = dist_amount.max(0.0);
            let one_plus_k = 1.0 + k;
            for frame in self.scratch[..n].iter_mut() {
                for c in 0..nch {
                    let x = frame[c];
                    frame[c] = (one_plus_k * x / (1.0 + k * x.abs())) * postgain;
                }
            }
        }

        // DC blocker: only if any distortion stage was active.
        if self.params.coarse.is_some()
            || self.params.crush.is_some()
            || self.params.fold.is_some()
            || self.params.wrap.is_some()
            || self.params.distort.is_some()
        {
            for c in 0..nch {
                self.dc_block[c].process_block(&mut self.scratch[..n], n, c);
            }
        }

        // AM modulation. LFO ticks per sample; depth/shape/rate are block-rate.
        if self.params.am > 0.0 {
            let depth = self.params.amdepth.clamp(0.0, 1.0);
            let shape = self.params.amshape;
            let rate = self.params.am;
            for frame in self.scratch[..n].iter_mut() {
                let modulator = self.am_lfo.lfo(shape, rate, isr);
                let factor = 1.0 + modulator * depth;
                for c in 0..nch {
                    frame[c] *= factor;
                }
            }
        }

        // Ring modulation.
        if self.params.rm > 0.0 {
            let depth = self.params.rmdepth.clamp(0.0, 1.0);
            let shape = self.params.rmshape;
            let rate = self.params.rm;
            let one_minus_depth = 1.0 - depth;
            for frame in self.scratch[..n].iter_mut() {
                let modulator = self.rm_lfo.lfo(shape, rate, isr);
                let factor = one_minus_depth + modulator * depth;
                for c in 0..nch {
                    frame[c] *= factor;
                }
            }
        }

        // Phaser.
        if self.params.phaser > 0.0 {
            let rate = self.params.phaser;
            let depth = self.params.phaserdepth;
            let center = self.params.phasercenter;
            let sweep = self.params.phasersweep;
            for c in 0..nch {
                self.phaser[c].process_block(
                    &mut self.scratch[..n],
                    n,
                    c,
                    rate,
                    depth,
                    center,
                    sweep,
                    sr,
                    isr,
                );
            }
        }

        // Flanger (pre-allocated via `ensure_effects`).
        if self.params.flanger > 0.0 {
            if let Some(flanger) = self.flanger.as_mut() {
                let rate = self.params.flanger;
                let depth = self.params.flangerdepth;
                let fb = self.params.flangerfeedback;
                for c in 0..nch {
                    flanger[c].process_block(
                        &mut self.scratch[..n],
                        n,
                        c,
                        rate,
                        depth,
                        fb,
                        sr,
                        isr,
                    );
                }
            }
        }

        // EQ.
        if self.params.eqlo != 0.0 || self.params.eqmid != 0.0 || self.params.eqhi != 0.0 {
            let lo_db = self.params.eqlo;
            let mid_db = self.params.eqmid;
            let hi_db = self.params.eqhi;
            let lo_freq = self.params.eqlofreq;
            let mid_freq = self.params.eqmidfreq;
            let hi_freq = self.params.eqhifreq;
            for c in 0..nch {
                self.eq[c].process_block(
                    &mut self.scratch[..n],
                    n,
                    c,
                    lo_db,
                    mid_db,
                    hi_db,
                    lo_freq,
                    mid_freq,
                    hi_freq,
                    sr,
                );
            }
        }

        // Tilt.
        if self.params.tilt != 0.0 {
            let tilt_amt = self.params.tilt;
            for c in 0..nch {
                self.tilt[c].process_block(&mut self.scratch[..n], n, c, tilt_amt, sr);
            }
        }

        // Smear.
        if self.params.smear > 0.0 {
            let mix = self.params.smear;
            let freq = self.params.smearfreq;
            let fb = self.params.smearfb;
            for c in 0..nch {
                self.smear[c].process_block(&mut self.scratch[..n], n, c, mix, freq, fb, sr);
            }
        }

        // VCA: envelope × postgain × velocity. Envelope is per-sample; postgain
        // and velocity are block-rate.
        let gain_block = self.params.postgain * self.params.velocity;
        for (i, frame) in self.scratch[..n].iter_mut().enumerate() {
            let voice_gain = env_buf[i] * gain_block;
            for c in 0..nch {
                frame[c] *= voice_gain;
            }
        }

        // Mono sources: spread or duplicate to stereo. The spread side-signal
        // is scaled by per-sample voice_gain (matches legacy ordering: spread
        // applied AFTER VCA, so the side amplitude tracks the envelope).
        if nch == 1 {
            if self.params.spread > 0.0 {
                let spread_side = self.spread_side;
                for (i, frame) in self.scratch[..n].iter_mut().enumerate() {
                    let voice_gain = env_buf[i] * gain_block;
                    let side = spread_side * voice_gain;
                    frame[1] = frame[0] - side;
                    frame[0] += side;
                }
            } else {
                for frame in self.scratch[..n].iter_mut() {
                    frame[1] = frame[0];
                }
            }
        }

        // Chorus (pre-allocated via `ensure_effects`).
        if self.params.chorus > 0.0 {
            if let Some(chorus) = self.chorus.as_mut() {
                let rate = self.params.chorus;
                let depth = self.params.chorusdepth;
                let delay_ms = self.params.chorusdelay;
                for frame in self.scratch[..n].iter_mut() {
                    let stereo = chorus.process(frame[0], frame[1], rate, depth, delay_ms, sr, isr);
                    frame[0] = stereo[0];
                    frame[1] = stereo[1];
                }
            }
        }

        // Stereo width (mid-side matrix).
        if self.params.width != 1.0 {
            let w = self.params.width.max(0.0);
            for frame in self.scratch[..n].iter_mut() {
                let mid = (frame[0] + frame[1]) * 0.5;
                let side = (frame[0] - frame[1]) * 0.5;
                frame[0] = mid + side * w;
                frame[1] = mid - side * w;
            }
        }

        // Haas (pre-allocated via `ensure_effects`).
        if self.params.haas > 0.0 {
            if let Some(haas) = self.haas.as_mut() {
                let ms = self.params.haas;
                for frame in self.scratch[..n].iter_mut() {
                    frame[1] = haas.process(frame[1], ms, sr);
                }
            }
        }

        // Panning.
        if self.params.pan != 0.5 {
            let pan_pos = self.params.pan * PI / 2.0;
            let l = cosf(pan_pos);
            let r = sinf(pan_pos);
            for frame in self.scratch[..n].iter_mut() {
                frame[0] *= l;
                frame[1] *= r;
            }
        }

        // Output trim.
        for frame in self.scratch[..n].iter_mut() {
            for c in 0..CHANNELS {
                frame[c] *= VOICE_OUTPUT_TRIM;
            }
        }
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
