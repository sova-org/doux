//! Voice - the core synthesis unit.

mod drums;
pub mod modulation;
mod params;
mod pluck;
mod source;

pub use modulation::{ModChain, ParamId, ParamMod};
pub use params::VoiceParams;
use pluck::PluckState;

use std::f32::consts::PI;

use crate::dsp::{cosf, exp2f, sinf, BrownNoise, Dahdsr, Phasor, PinkNoise, SvfMode};
use crate::effects::{
    DcBlocker, FaustChorus, FaustCoarse, FaustCrush, FaustDistort, FaustEq, FaustFlanger,
    FaustFold, FaustHaas, FaustLadder, FaustPhaser, FaustSmear, FaustSvf, FaustSvfCascade,
    FaustTilt, FaustWrap, LadderMode,
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
const VOICE_OUTPUT_TRIM: f32 = 0.5;
/// `1 / (2π)`: converts radians to turns for phase-modulation math.
const INV_TAU: f32 = 0.159_154_94;

/// Per-sample voice stages, in execution order. The block-rate program
/// (`Voice::stage_program`) is a packed list of exactly the stages active
/// for the current block; the per-sample executor runs them in array order.
///
/// Voice-core stages (`PreGain`, `Vca`, `MonoStereo`, `Width`, `Pan`, `Trim`)
/// always emit. FX stages conditionally emit based on
/// `(param is set/non-default) OR (a ParamMod targets the stage's gate
/// param)`. `DcBlock` emits iff any of the distortion-class stages
/// (`Coarse`/`Crush`/`Fold`/`Wrap`/`Distort`) emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Stage {
    // Pre-VCA — voice-core
    PreGain,
    // Pre-VCA — FX
    Lpf,
    Hpf,
    Bpf,
    SteepLpf,
    SteepHpf,
    SteepBpf,
    LadderLp,
    LadderHp,
    LadderBp,
    Coarse,
    Crush,
    Fold,
    Wrap,
    Distort,
    DcBlock,
    Am,
    Rm,
    Phaser,
    Flanger,
    Eq,
    Tilt,
    Smear,
    // VCA + stereo finalize — voice-core (except chorus/haas)
    Vca,
    MonoStereo,
    Chorus,
    Width,
    Haas,
    Pan,
    Trim,
}

/// Upper bound on stages a voice can emit per block. Sized to the count of
/// `Stage` variants with room to spare; allocated inline on `Voice`.
pub(crate) const MAX_STAGES: usize = 32;

/// Cold FX state — pre-allocated, heap-owned via [`Voice::fx`]. Pulling it
/// out of [`Voice`] keeps the hot-voice working set inside L1d when scanning
/// `active_voices` during the per-block kernel. One indirection per FX
/// stage per block; per-sample inner loops still touch only their own field.
pub struct VoiceFxState {
    pub slp: [FaustSvfCascade; CHANNELS],
    pub shp: [FaustSvfCascade; CHANNELS],
    pub sbp: [FaustSvfCascade; CHANNELS],
    pub ladder_lp: [FaustLadder; CHANNELS],
    pub ladder_hp: [FaustLadder; CHANNELS],
    pub ladder_bp: [FaustLadder; CHANNELS],
    pub coarse: [FaustCoarse; CHANNELS],
    pub crush: [FaustCrush; CHANNELS],
    pub fold_state: [FaustFold; CHANNELS],
    pub wrap_state: [FaustWrap; CHANNELS],
    pub distort_state: [FaustDistort; CHANNELS],
    pub dc_block: [DcBlocker; CHANNELS],
    pub eq: [FaustEq; CHANNELS],
    pub tilt: [FaustTilt; CHANNELS],
    pub phaser: [FaustPhaser; CHANNELS],
    pub flanger: [FaustFlanger; CHANNELS],
    pub smear: [FaustSmear; CHANNELS],
    pub chorus: FaustChorus,
    pub haas: FaustHaas,
}

impl Default for VoiceFxState {
    fn default() -> Self {
        Self {
            slp: std::array::from_fn(|_| FaustSvfCascade::default()),
            shp: std::array::from_fn(|_| FaustSvfCascade::default()),
            sbp: std::array::from_fn(|_| FaustSvfCascade::default()),
            ladder_lp: std::array::from_fn(|_| FaustLadder::default()),
            ladder_hp: std::array::from_fn(|_| FaustLadder::default()),
            ladder_bp: std::array::from_fn(|_| FaustLadder::default()),
            coarse: std::array::from_fn(|_| FaustCoarse::default()),
            crush: std::array::from_fn(|_| FaustCrush::default()),
            fold_state: std::array::from_fn(|_| FaustFold::default()),
            wrap_state: std::array::from_fn(|_| FaustWrap::default()),
            distort_state: std::array::from_fn(|_| FaustDistort::default()),
            dc_block: [DcBlocker::default(); CHANNELS],
            eq: std::array::from_fn(|_| FaustEq::default()),
            tilt: std::array::from_fn(|_| FaustTilt::default()),
            phaser: std::array::from_fn(FaustPhaser::new),
            flanger: std::array::from_fn(FaustFlanger::new),
            smear: std::array::from_fn(|_| FaustSmear::default()),
            chorus: FaustChorus::default(),
            haas: FaustHaas::default(),
        }
    }
}

/// Layout: hot fields (touched every block, every active voice) cluster on
/// `Voice`; cold FX state lives behind [`Voice::fx`] (heap-allocated) so the
/// hot working set stays inside L1d during the per-voice block kernel.
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
    /// Computed once per sample by `tick_fm_pm`; read by `generate_main_osc`.
    pub fm_phase_mod: f32,
    pub am_lfo: Phasor,
    pub rm_lfo: Phasor,
    pub current_freq: f32,
    pub lp: [FaustSvf; CHANNELS],
    pub hp: [FaustSvf; CHANNELS],
    pub bp: [FaustSvf; CHANNELS],
    pub nch: usize,
    pub spread_cache_value: f32,
    pub spread_detune_ratios: [f32; 3],
    pub triggered: bool,
    pub time: f32,
    pub sr: f32,
    pub seed: u32,
    /// Stable identity for event addressing (`voice/N`). Survives the
    /// swap-remove in voice freeing because the whole `Voice` moves.
    pub tag: Option<usize>,
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
    pub(super) drum_svf: FaustSvf,
    pub(super) drum_svf2: FaustSvf,
    /// Karplus-Strong state for the `pluck` source. Boxed (~32 KB delay line)
    /// and allocated once at construction — never on the audio thread.
    pub(super) pluck: Box<PluckState>,

    // === Param modulation (read once per block in `apply_mods`) ===
    pub param_mods: [(ParamId, ParamMod); MAX_PARAM_MODS],
    pub param_mod_count: u8,

    /// Cold FX state — pre-allocated on the heap so the hot voice struct
    /// stays small. One indirection per FX stage per block; per-sample
    /// loops still operate on inline fields of `*self.fx`.
    pub fx: Box<VoiceFxState>,

    /// Per-block stage program. Built by [`Voice::build_stage_program`] at
    /// the top of each `run_source_block`; iterated by [`Voice::finish_sample`]
    /// once per sample. Only the first `stage_count` entries are valid.
    pub(crate) stage_program: [Stage; MAX_STAGES],
    pub(crate) stage_count: u8,
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
            lp: std::array::from_fn(|_| FaustSvf::default()),
            hp: std::array::from_fn(|_| FaustSvf::default()),
            bp: std::array::from_fn(|_| FaustSvf::default()),
            nch: 1,
            spread_cache_value: f32::NAN,
            spread_detune_ratios: [1.0; 3],
            triggered: false,
            time: 0.0,
            sr,
            seed: 123456789,
            tag: None,
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
            drum_svf: FaustSvf::default(),
            drum_svf2: FaustSvf::default(),
            pluck: Box::new(PluckState::default()),
            param_mods: [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS],
            param_mod_count: 0,
            fx: Box::new(VoiceFxState::default()),
            stage_program: [Stage::PreGain; MAX_STAGES],
            stage_count: 0,
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
        self.lp = std::array::from_fn(|_| FaustSvf::default());
        self.hp = std::array::from_fn(|_| FaustSvf::default());
        self.bp = std::array::from_fn(|_| FaustSvf::default());
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
        // Clear all cold FX state in one go — no reallocation.
        *self.fx = VoiceFxState::default();
        self.param_mods = [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS];
        self.param_mod_count = 0;
        self.triggered = false;
        self.time = 0.0;
        self.tag = None;
        *self.scratch = [[0.0; CHANNELS]; MAX_BLOCK];
        self.nch = 1;
        self.spread_cache_value = f32::NAN;
        self.spread_detune_ratios = [1.0; 3];
        self.shape_active = false;
        self.sr = 44100.0;
        self.seed = 123456789;
        self.drum_svf = FaustSvf::default();
        self.drum_svf2 = FaustSvf::default();
        // O(1): the 32 KB delay line is re-zeroed lazily by `run_pluck` on the
        // first sample of a pluck note, never here on every note-on.
        self.pluck.primed = false;
        self.stage_count = 0;
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
                let detune_cents = (i + 1) as f32 / 3.0 * self.params.spread;
                *ratio = exp2f(detune_cents / 1200.0);
            }
            self.spread_cache_value = self.params.spread;
        }
        &self.spread_detune_ratios
    }

    #[inline]
    pub(crate) fn sync_source_state(&mut self) {
        self.shape_active = self.params.shape.is_active();
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

    /// Remove any active ModChain targeting `id` (swap-remove, no alloc).
    /// The param keeps its last written value.
    pub fn clear_mod(&mut self, id: ParamId) {
        let mut i = 0;
        while i < self.param_mod_count as usize {
            if self.param_mods[i].0 == id {
                self.param_mod_count -= 1;
                self.param_mods.swap(i, self.param_mod_count as usize);
            } else {
                i += 1;
            }
        }
    }

    /// Re-fire the envelopes at the next `prepare_block` without resetting
    /// phase or params. `Dahdsr::trigger` ramps from `current_val`, so a
    /// retrigger on a sounding voice is click-free.
    pub fn retrigger(&mut self) {
        self.triggered = false;
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
            ParamId::EqMidQ => self.params.eqmidq,
            ParamId::EqHiFreq => self.params.eqhifreq,
            ParamId::Superpan => self.params.superpan.unwrap_or(0.0),
            ParamId::Superwidth => self.params.superwidth,
        }
    }

    /// True if any active `ParamMod` targets `id` (the stage's gate param).
    #[inline]
    fn mod_targets(&self, id: ParamId) -> bool {
        for k in 0..self.param_mod_count as usize {
            if self.param_mods[k].0 == id {
                return true;
            }
        }
        false
    }

    /// Build the per-block stage program: a packed list of exactly the
    /// stages this voice needs this block. Called once per block before the
    /// per-sample source loop; iterated by [`Voice::finish_sample`].
    ///
    /// Predicate for each FX stage: `(gate currently on) || mod_targets(gate)`.
    /// `DcBlock` follows from any of the distortion stages emitting. Voice-core
    /// stages (`PreGain`, `Vca`, `MonoStereo`, `Width`, `Pan`, `Trim`) always
    /// emit; their per-sample bodies still default-gate `width != 1` /
    /// `pan != 0.5` / `spread > 0`, so default values are essentially free.
    pub(crate) fn build_stage_program(&mut self) {
        let coarse = self.params.coarse.is_some() || self.mod_targets(ParamId::Coarse);
        let crush = self.params.crush.is_some() || self.mod_targets(ParamId::Crush);
        let fold = self.params.fold.is_some() || self.mod_targets(ParamId::Fold);
        let wrap = self.params.wrap.is_some() || self.mod_targets(ParamId::Wrap);
        let distort = self.params.distort.is_some() || self.mod_targets(ParamId::Distort);
        let any_dist = coarse || crush || fold || wrap || distort;

        let mut count = 0_u8;
        macro_rules! push {
            ($s:expr) => {{
                self.stage_program[count as usize] = $s;
                count += 1;
            }};
        }

        push!(Stage::PreGain);

        if self.params.lpf.is_some() || self.mod_targets(ParamId::Lpf) {
            push!(Stage::Lpf);
        }
        if self.params.hpf.is_some() || self.mod_targets(ParamId::Hpf) {
            push!(Stage::Hpf);
        }
        if self.params.bpf.is_some() || self.mod_targets(ParamId::Bpf) {
            push!(Stage::Bpf);
        }
        if self.params.slpf.is_some() || self.mod_targets(ParamId::Slpf) {
            push!(Stage::SteepLpf);
        }
        if self.params.shpf.is_some() || self.mod_targets(ParamId::Shpf) {
            push!(Stage::SteepHpf);
        }
        if self.params.sbpf.is_some() || self.mod_targets(ParamId::Sbpf) {
            push!(Stage::SteepBpf);
        }
        if self.params.llpf.is_some() || self.mod_targets(ParamId::Llpf) {
            push!(Stage::LadderLp);
        }
        if self.params.lhpf.is_some() || self.mod_targets(ParamId::Lhpf) {
            push!(Stage::LadderHp);
        }
        if self.params.lbpf.is_some() || self.mod_targets(ParamId::Lbpf) {
            push!(Stage::LadderBp);
        }

        if coarse {
            push!(Stage::Coarse);
        }
        if crush {
            push!(Stage::Crush);
        }
        if fold {
            push!(Stage::Fold);
        }
        if wrap {
            push!(Stage::Wrap);
        }
        if distort {
            push!(Stage::Distort);
        }
        if any_dist {
            push!(Stage::DcBlock);
        }

        if self.params.am > 0.0 || self.mod_targets(ParamId::Am) {
            push!(Stage::Am);
        }
        if self.params.rm > 0.0 || self.mod_targets(ParamId::Rm) {
            push!(Stage::Rm);
        }
        if self.params.phaser > 0.0 || self.mod_targets(ParamId::Phaser) {
            push!(Stage::Phaser);
        }
        if self.params.flanger > 0.0 || self.mod_targets(ParamId::Flanger) {
            push!(Stage::Flanger);
        }
        if self.params.eqlo != 0.0
            || self.params.eqmid != 0.0
            || self.params.eqhi != 0.0
            || self.mod_targets(ParamId::Eqlo)
            || self.mod_targets(ParamId::Eqmid)
            || self.mod_targets(ParamId::Eqhi)
        {
            push!(Stage::Eq);
        }
        if self.params.tilt != 0.0 || self.mod_targets(ParamId::Tilt) {
            push!(Stage::Tilt);
        }
        if self.params.smear > 0.0 || self.mod_targets(ParamId::Smear) {
            push!(Stage::Smear);
        }

        push!(Stage::Vca);
        push!(Stage::MonoStereo);
        if self.params.chorus > 0.0 || self.mod_targets(ParamId::Chorus) {
            push!(Stage::Chorus);
        }
        push!(Stage::Width);
        if self.params.haas > 0.0 || self.mod_targets(ParamId::Haas) {
            push!(Stage::Haas);
        }
        push!(Stage::Pan);
        push!(Stage::Trim);

        self.stage_count = count;
    }

    /// Per-sample executor: iterates `self.stage_program[..stage_count]` and
    /// dispatches each entry through [`Voice::tick_stage`].
    ///
    /// Early-exits when `param_mod_count == 0`; that path is handled by
    /// [`Voice::finish_block`] after the source loop closes, which runs the
    /// same DSP in block-rate dispatch. With no active mods the two paths
    /// are mathematically equivalent (filter state machines are identical;
    /// only loop order changes).
    #[inline]
    pub(crate) fn finish_sample(&mut self, env: f32, isr: f32, i: usize) {
        if self.param_mod_count == 0 {
            return;
        }
        let sr = self.sr;
        for k in 0..self.stage_count as usize {
            let stage = self.stage_program[k];
            self.tick_stage(stage, i, env, sr, isr);
        }
    }

    /// Block-rate executor: runs each stage in `stage_program[..stage_count]`
    /// once over `scratch[..n]`, dispatching to per-stage `process_block`
    /// APIs. Called from `run_source_block` after the source body has filled
    /// `scratch[..n]`.
    ///
    /// Only active when `param_mod_count == 0`. Per-sample `apply_mods_one`
    /// would mutate `params` mid-block, so when mods are active the per-sample
    /// executor in [`Voice::finish_sample`] runs interleaved with the source
    /// loop. With no mods `params` is stable for the block; block-rate
    /// dispatch reads each param once at block entry, amortizes coefficient
    /// recompute, and lets the compiler vectorize the per-stage inner loop.
    pub(crate) fn finish_block(&mut self, env: &[f32], n: usize, isr: f32) {
        if self.param_mod_count > 0 {
            return;
        }
        let sr = self.sr;
        let nch = self.nch;
        for k in 0..self.stage_count as usize {
            let stage = self.stage_program[k];
            self.tick_stage_block(stage, env, n, sr, isr, nch);
        }
    }

    /// One block of one stage on `self.scratch[..n]`. Mirrors `tick_stage`
    /// arms; reads each param once at block entry; inner loop is straight-
    /// line state update.
    #[allow(clippy::needless_range_loop, clippy::too_many_arguments)]
    fn tick_stage_block(
        &mut self,
        stage: Stage,
        env: &[f32],
        n: usize,
        sr: f32,
        isr: f32,
        nch: usize,
    ) {
        match stage {
            Stage::PreGain => {
                let gain = self.params.gain;
                for i in 0..n {
                    for c in 0..nch {
                        self.scratch[i][c] *= gain;
                    }
                }
            }
            Stage::Lpf => {
                if let Some(lpf) = self.params.lpf {
                    let q = self.params.lpq;
                    for c in 0..nch {
                        self.lp[c].cutoff = lpf;
                        self.lp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Lp, q, sr);
                    }
                }
            }
            Stage::Hpf => {
                if let Some(hpf) = self.params.hpf {
                    let q = self.params.hpq;
                    for c in 0..nch {
                        self.hp[c].cutoff = hpf;
                        self.hp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Hp, q, sr);
                    }
                }
            }
            Stage::Bpf => {
                if let Some(bpf) = self.params.bpf {
                    let q = self.params.bpq;
                    for c in 0..nch {
                        self.bp[c].cutoff = bpf;
                        self.bp[c].process_block(&mut self.scratch[..n], n, c, SvfMode::Bp, q, sr);
                    }
                }
            }
            Stage::SteepLpf => {
                if let Some(slpf) = self.params.slpf {
                    let q = self.params.slpq;
                    for c in 0..nch {
                        self.fx.slp[c].cutoff = slpf;
                        self.fx.slp[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            SvfMode::Lp,
                            q,
                            sr,
                        );
                    }
                }
            }
            Stage::SteepHpf => {
                if let Some(shpf) = self.params.shpf {
                    let q = self.params.shpq;
                    for c in 0..nch {
                        self.fx.shp[c].cutoff = shpf;
                        self.fx.shp[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            SvfMode::Hp,
                            q,
                            sr,
                        );
                    }
                }
            }
            Stage::SteepBpf => {
                if let Some(sbpf) = self.params.sbpf {
                    let q = self.params.sbpq;
                    for c in 0..nch {
                        self.fx.sbp[c].cutoff = sbpf;
                        self.fx.sbp[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            SvfMode::Bp,
                            q,
                            sr,
                        );
                    }
                }
            }
            Stage::LadderLp => {
                if let Some(llpf) = self.params.llpf {
                    let q = self.params.llpq;
                    for c in 0..nch {
                        self.fx.ladder_lp[c].process_block(
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
            }
            Stage::LadderHp => {
                if let Some(lhpf) = self.params.lhpf {
                    let q = self.params.lhpq;
                    for c in 0..nch {
                        self.fx.ladder_hp[c].process_block(
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
            }
            Stage::LadderBp => {
                if let Some(lbpf) = self.params.lbpf {
                    let q = self.params.lbpq;
                    for c in 0..nch {
                        self.fx.ladder_bp[c].process_block(
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
            }
            Stage::Coarse => {
                if let Some(factor) = self.params.coarse {
                    for c in 0..nch {
                        self.fx.coarse[c].process_block(&mut self.scratch[..n], n, c, factor);
                    }
                }
            }
            Stage::Crush => {
                if let Some(crush_bits) = self.params.crush {
                    for c in 0..nch {
                        self.fx.crush[c].process_block(&mut self.scratch[..n], n, c, crush_bits);
                    }
                }
            }
            Stage::Fold => {
                if let Some(amount) = self.params.fold {
                    let mode = self.params.foldmode.to_index();
                    for c in 0..nch {
                        self.fx.fold_state[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            amount,
                            mode,
                        );
                    }
                }
            }
            Stage::Wrap => {
                if let Some(amount) = self.params.wrap {
                    for c in 0..nch {
                        self.fx.wrap_state[c].process_block(&mut self.scratch[..n], n, c, amount);
                    }
                }
            }
            Stage::Distort => {
                if let Some(amount) = self.params.distort {
                    let postgain = self.params.distortvol;
                    let mode = self.params.distortmode.to_index();
                    for c in 0..nch {
                        self.fx.distort_state[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            amount,
                            postgain,
                            mode,
                        );
                    }
                }
            }
            Stage::DcBlock => {
                for c in 0..nch {
                    self.fx.dc_block[c].process_block(&mut self.scratch[..n], n, c);
                }
            }
            Stage::Am => {
                if self.params.am > 0.0 {
                    let depth = self.params.amdepth.clamp(0.0, 1.0);
                    let am = self.params.am;
                    let shape = self.params.amshape;
                    for i in 0..n {
                        let modulator = self.am_lfo.lfo(shape, am, isr);
                        let factor = 1.0 + modulator * depth;
                        for c in 0..nch {
                            self.scratch[i][c] *= factor;
                        }
                    }
                }
            }
            Stage::Rm => {
                if self.params.rm > 0.0 {
                    let depth = self.params.rmdepth.clamp(0.0, 1.0);
                    let rm = self.params.rm;
                    let shape = self.params.rmshape;
                    for i in 0..n {
                        let modulator = self.rm_lfo.lfo(shape, rm, isr);
                        let factor = (1.0 - depth) + modulator * depth;
                        for c in 0..nch {
                            self.scratch[i][c] *= factor;
                        }
                    }
                }
            }
            Stage::Phaser => {
                if self.params.phaser > 0.0 {
                    let rate = self.params.phaser;
                    let depth = self.params.phaserdepth;
                    let center = self.params.phasercenter;
                    let sweep = self.params.phasersweep;
                    for c in 0..nch {
                        self.fx.phaser[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            rate,
                            depth,
                            center,
                            sweep,
                            sr,
                        );
                    }
                }
            }
            Stage::Flanger => {
                if self.params.flanger > 0.0 {
                    let rate = self.params.flanger;
                    let depth = self.params.flangerdepth;
                    let fb = self.params.flangerfeedback;
                    for c in 0..nch {
                        self.fx.flanger[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            rate,
                            depth,
                            fb,
                            sr,
                        );
                    }
                }
            }
            Stage::Eq => {
                if self.params.eqlo != 0.0 || self.params.eqmid != 0.0 || self.params.eqhi != 0.0 {
                    let lo_db = self.params.eqlo;
                    let mid_db = self.params.eqmid;
                    let hi_db = self.params.eqhi;
                    let lo_freq = self.params.eqlofreq;
                    let mid_freq = self.params.eqmidfreq;
                    let hi_freq = self.params.eqhifreq;
                    let mid_q = self.params.eqmidq;
                    for c in 0..nch {
                        self.fx.eq[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            lo_db,
                            mid_db,
                            hi_db,
                            lo_freq,
                            mid_freq,
                            hi_freq,
                            mid_q,
                            sr,
                        );
                    }
                }
            }
            Stage::Tilt => {
                if self.params.tilt != 0.0 {
                    let tilt_amt = self.params.tilt;
                    for c in 0..nch {
                        self.fx.tilt[c].process_block(&mut self.scratch[..n], n, c, tilt_amt, sr);
                    }
                }
            }
            Stage::Smear => {
                if self.params.smear > 0.0 {
                    let mix = self.params.smear;
                    let freq = self.params.smearfreq;
                    let fb = self.params.smearfb;
                    for c in 0..nch {
                        self.fx.smear[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            mix,
                            freq,
                            fb,
                            sr,
                        );
                    }
                }
            }
            Stage::Vca => {
                let base = self.params.postgain * self.params.velocity;
                for i in 0..n {
                    let g = env[i] * base;
                    for c in 0..nch {
                        self.scratch[i][c] *= g;
                    }
                }
            }
            Stage::MonoStereo => {
                if nch == 1 {
                    if self.params.spread > 0.0 {
                        let base = self.params.postgain * self.params.velocity;
                        for i in 0..n {
                            let mid = self.scratch[i][0];
                            let side = env[i] * base * self.scratch[i][1];
                            self.scratch[i][0] = mid + side;
                            self.scratch[i][1] = mid - side;
                        }
                    } else {
                        for i in 0..n {
                            self.scratch[i][1] = self.scratch[i][0];
                        }
                    }
                }
            }
            Stage::Chorus => {
                if self.params.chorus > 0.0 {
                    let rate = self.params.chorus;
                    let depth = self.params.chorusdepth;
                    let delay_ms = self.params.chorusdelay;
                    let ctype = self.params.chorustype.to_index();
                    self.fx.chorus.process_block(
                        &mut self.scratch[..n],
                        n,
                        rate,
                        depth,
                        delay_ms,
                        ctype,
                        sr,
                    );
                }
            }
            Stage::Width => {
                if self.params.width != 1.0 {
                    let w = self.params.width.max(0.0);
                    for i in 0..n {
                        let mid = (self.scratch[i][0] + self.scratch[i][1]) * 0.5;
                        let side = (self.scratch[i][0] - self.scratch[i][1]) * 0.5;
                        self.scratch[i][0] = mid + side * w;
                        self.scratch[i][1] = mid - side * w;
                    }
                }
            }
            Stage::Haas => {
                if self.params.haas > 0.0 {
                    let ms = self.params.haas;
                    self.fx
                        .haas
                        .process_block(&mut self.scratch[..n], n, ms, sr);
                }
            }
            Stage::Pan => {
                if self.params.pan != 0.5 {
                    let pan_pos = self.params.pan * PI / 2.0;
                    let l = cosf(pan_pos);
                    let r = sinf(pan_pos);
                    for i in 0..n {
                        self.scratch[i][0] *= l;
                        self.scratch[i][1] *= r;
                    }
                }
            }
            Stage::Trim => {
                for i in 0..n {
                    for c in 0..CHANNELS {
                        self.scratch[i][c] *= VOICE_OUTPUT_TRIM;
                    }
                }
            }
        }
    }

    /// One sample of one stage on `self.scratch[i]`. Match-on-`Stage` compiles
    /// to a jump table; the dispatch sequence is identical for every sample
    /// of a given block, so the indirect branch predicts cleanly.
    #[inline]
    #[allow(clippy::needless_range_loop)]
    fn tick_stage(&mut self, stage: Stage, i: usize, env: f32, sr: f32, isr: f32) {
        let nch = self.nch;
        match stage {
            Stage::PreGain => {
                let gain = self.params.gain;
                for c in 0..nch {
                    self.scratch[i][c] *= gain;
                }
            }
            Stage::Lpf => {
                if let Some(lpf) = self.params.lpf {
                    let q = self.params.lpq;
                    for c in 0..nch {
                        self.lp[c].cutoff = lpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.lp[c].process(x, SvfMode::Lp, q, sr);
                    }
                }
            }
            Stage::Hpf => {
                if let Some(hpf) = self.params.hpf {
                    let q = self.params.hpq;
                    for c in 0..nch {
                        self.hp[c].cutoff = hpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.hp[c].process(x, SvfMode::Hp, q, sr);
                    }
                }
            }
            Stage::Bpf => {
                if let Some(bpf) = self.params.bpf {
                    let q = self.params.bpq;
                    for c in 0..nch {
                        self.bp[c].cutoff = bpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.bp[c].process(x, SvfMode::Bp, q, sr);
                    }
                }
            }
            Stage::SteepLpf => {
                if let Some(slpf) = self.params.slpf {
                    let q = self.params.slpq;
                    for c in 0..nch {
                        self.fx.slp[c].cutoff = slpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.slp[c].process(x, SvfMode::Lp, q, sr);
                    }
                }
            }
            Stage::SteepHpf => {
                if let Some(shpf) = self.params.shpf {
                    let q = self.params.shpq;
                    for c in 0..nch {
                        self.fx.shp[c].cutoff = shpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.shp[c].process(x, SvfMode::Hp, q, sr);
                    }
                }
            }
            Stage::SteepBpf => {
                if let Some(sbpf) = self.params.sbpf {
                    let q = self.params.sbpq;
                    for c in 0..nch {
                        self.fx.sbp[c].cutoff = sbpf;
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.sbp[c].process(x, SvfMode::Bp, q, sr);
                    }
                }
            }
            Stage::LadderLp => {
                if let Some(llpf) = self.params.llpf {
                    let q = self.params.llpq;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.ladder_lp[c].process(x, llpf, q, LadderMode::Lp, sr);
                    }
                }
            }
            Stage::LadderHp => {
                if let Some(lhpf) = self.params.lhpf {
                    let q = self.params.lhpq;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.ladder_hp[c].process(x, lhpf, q, LadderMode::Hp, sr);
                    }
                }
            }
            Stage::LadderBp => {
                if let Some(lbpf) = self.params.lbpf {
                    let q = self.params.lbpq;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.ladder_bp[c].process(x, lbpf, q, LadderMode::Bp, sr);
                    }
                }
            }
            Stage::Coarse => {
                if let Some(coarse_factor) = self.params.coarse {
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.coarse[c].process(x, coarse_factor);
                    }
                }
            }
            Stage::Crush => {
                if let Some(crush_bits) = self.params.crush {
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.crush[c].process(x, crush_bits);
                    }
                }
            }
            Stage::Fold => {
                if let Some(fold_amount) = self.params.fold {
                    let mode = self.params.foldmode.to_index();
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.fold_state[c].process(x, fold_amount, mode);
                    }
                }
            }
            Stage::Wrap => {
                if let Some(wrap_amount) = self.params.wrap {
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.wrap_state[c].process(x, wrap_amount);
                    }
                }
            }
            Stage::Distort => {
                if let Some(dist_amount) = self.params.distort {
                    let postgain = self.params.distortvol;
                    let mode = self.params.distortmode.to_index();
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.distort_state[c].process(x, dist_amount, postgain, mode);
                    }
                }
            }
            Stage::DcBlock => {
                // Only meaningful when a distortion stage actually ran on this
                // sample. Cheap enough to always run when emitted; the cost
                // is one IIR step per channel.
                for c in 0..nch {
                    let x = self.scratch[i][c];
                    self.scratch[i][c] = self.fx.dc_block[c].process(x);
                }
            }
            Stage::Am => {
                if self.params.am > 0.0 {
                    let depth = self.params.amdepth.clamp(0.0, 1.0);
                    let modulator = self.am_lfo.lfo(self.params.amshape, self.params.am, isr);
                    let factor = 1.0 + modulator * depth;
                    for c in 0..nch {
                        self.scratch[i][c] *= factor;
                    }
                }
            }
            Stage::Rm => {
                if self.params.rm > 0.0 {
                    let depth = self.params.rmdepth.clamp(0.0, 1.0);
                    let modulator = self.rm_lfo.lfo(self.params.rmshape, self.params.rm, isr);
                    let factor = (1.0 - depth) + modulator * depth;
                    for c in 0..nch {
                        self.scratch[i][c] *= factor;
                    }
                }
            }
            Stage::Phaser => {
                if self.params.phaser > 0.0 {
                    let rate = self.params.phaser;
                    let depth = self.params.phaserdepth;
                    let center = self.params.phasercenter;
                    let sweep = self.params.phasersweep;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.phaser[c].process(x, rate, depth, center, sweep, sr);
                    }
                }
            }
            Stage::Flanger => {
                if self.params.flanger > 0.0 {
                    let rate = self.params.flanger;
                    let depth = self.params.flangerdepth;
                    let fb = self.params.flangerfeedback;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.flanger[c].process(x, rate, depth, fb, sr);
                    }
                }
            }
            Stage::Eq => {
                if self.params.eqlo != 0.0 || self.params.eqmid != 0.0 || self.params.eqhi != 0.0 {
                    let lo_db = self.params.eqlo;
                    let mid_db = self.params.eqmid;
                    let hi_db = self.params.eqhi;
                    let lo_freq = self.params.eqlofreq;
                    let mid_freq = self.params.eqmidfreq;
                    let hi_freq = self.params.eqhifreq;
                    let mid_q = self.params.eqmidq;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.eq[c].process(
                            x, lo_db, mid_db, hi_db, lo_freq, mid_freq, hi_freq, mid_q, sr,
                        );
                    }
                }
            }
            Stage::Tilt => {
                if self.params.tilt != 0.0 {
                    let tilt_amt = self.params.tilt;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.tilt[c].process(x, tilt_amt, sr);
                    }
                }
            }
            Stage::Smear => {
                if self.params.smear > 0.0 {
                    let mix = self.params.smear;
                    let freq = self.params.smearfreq;
                    let fb = self.params.smearfb;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.smear[c].process(x, mix, freq, fb, sr);
                    }
                }
            }
            Stage::Vca => {
                let voice_gain = env * self.params.postgain * self.params.velocity;
                for c in 0..nch {
                    self.scratch[i][c] *= voice_gain;
                }
            }
            Stage::MonoStereo => {
                if nch == 1 {
                    if self.params.spread > 0.0 {
                        let voice_gain = env * self.params.postgain * self.params.velocity;
                        let mid = self.scratch[i][0];
                        let side = voice_gain * self.scratch[i][1];
                        self.scratch[i][0] = mid + side;
                        self.scratch[i][1] = mid - side;
                    } else {
                        self.scratch[i][1] = self.scratch[i][0];
                    }
                }
            }
            Stage::Chorus => {
                if self.params.chorus > 0.0 {
                    let rate = self.params.chorus;
                    let depth = self.params.chorusdepth;
                    let delay_ms = self.params.chorusdelay;
                    let ctype = self.params.chorustype.to_index();
                    let stereo = self.fx.chorus.process(
                        self.scratch[i][0],
                        self.scratch[i][1],
                        rate,
                        depth,
                        delay_ms,
                        ctype,
                        sr,
                    );
                    self.scratch[i][0] = stereo[0];
                    self.scratch[i][1] = stereo[1];
                }
            }
            Stage::Width => {
                if self.params.width != 1.0 {
                    let w = self.params.width.max(0.0);
                    let mid = (self.scratch[i][0] + self.scratch[i][1]) * 0.5;
                    let side = (self.scratch[i][0] - self.scratch[i][1]) * 0.5;
                    self.scratch[i][0] = mid + side * w;
                    self.scratch[i][1] = mid - side * w;
                }
            }
            Stage::Haas => {
                if self.params.haas > 0.0 {
                    let ms = self.params.haas;
                    self.scratch[i][1] = self.fx.haas.process(self.scratch[i][1], ms, sr);
                }
            }
            Stage::Pan => {
                if self.params.pan != 0.5 {
                    let pan_pos = self.params.pan * PI / 2.0;
                    let l = cosf(pan_pos);
                    let r = sinf(pan_pos);
                    self.scratch[i][0] *= l;
                    self.scratch[i][1] *= r;
                }
            }
            Stage::Trim => {
                for c in 0..CHANNELS {
                    self.scratch[i][c] *= VOICE_OUTPUT_TRIM;
                }
            }
        }
    }

    /// Sample-rate param-mod application: ticks each `ParamMod` once and writes
    /// the result to its target. Called once per sample inside the source loop.
    #[inline]
    fn apply_mods_one(&mut self, isr: f32) {
        for i in 0..self.param_mod_count as usize {
            let (id, ref mut m) = self.param_mods[i];
            let val = m.tick(isr);
            self.write_param(id, val);
        }
    }

    /// Per-sample voice "pre-source" tick: apply param-mods, tick the FM
    /// modulator phasors at the pre-vib carrier, advance the vib LFO. Returns
    /// the post-vib carrier freq for the source body to use.
    #[inline]
    pub(crate) fn tick_pre(&mut self, isr: f32) -> f32 {
        if self.param_mod_count > 0 {
            self.apply_mods_one(isr);
        }
        let pre_vib = self.fm_carrier_freq();
        self.tick_fm_pm(pre_vib, isr);
        self.compute_freq_one(pre_vib, isr)
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
            ParamId::Harmonics => self.params.harmonics = val,
            ParamId::Timbre => self.params.timbre = val,
            ParamId::Morph => self.params.morph = val,
            ParamId::Scan => self.params.scan = val,
            ParamId::Mirror => {
                self.params.shape.mirror = val;
                self.shape_active = self.params.shape.is_active();
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
            ParamId::EqMidQ => self.params.eqmidq = val,
            ParamId::EqHiFreq => self.params.eqhifreq = val,
            ParamId::Superpan => self.params.superpan = Some(val),
            ParamId::Superwidth => self.params.superwidth = val,
        }
    }

    /// Per-sample carrier frequency: takes the pre-vib carrier (detune ×
    /// speed, from [`Voice::fm_carrier_freq`] — computed once per sample by
    /// `tick_pre`), then ticks the vibrato LFO and applies `vibmod`. Stores
    /// the post-vib value in `self.current_freq` and returns it.
    ///
    /// FM phase modulation runs separately in [`Voice::tick_fm_pm`] and uses
    /// the same pre-vib carrier to preserve the legacy ordering
    /// `detune → speed → FM → vib`.
    #[inline]
    fn compute_freq_one(&mut self, pre_vib: f32, isr: f32) -> f32 {
        let mut freq = pre_vib;
        if self.params.vib > 0.0 && self.params.vibmod > 0.0 {
            let mod_val = self.vib_lfo.lfo(self.params.vibshape, self.params.vib, isr);
            freq *= exp2f(mod_val * self.params.vibmod / 12.0);
        }
        self.current_freq = freq;
        freq
    }

    /// Pre-vib carrier frequency for FM modulators. `detune × speed` only.
    /// Called once per sample by the source loop after `apply_mods_one`, so
    /// any per-sample modulation of `detune` or `speed` is honored.
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

    /// Block-rate preamble: trigger the envelope (once) and precompute `n`
    /// envelope samples into a stack-allocated buffer. Param-mods and carrier
    /// freq are computed per-sample inside the source loop, so they aren't
    /// touched here. Returns `None` if the envelope is `Off` after the block.
    pub(crate) fn prepare_block(&mut self, isr: f32, n: usize) -> Option<[f32; MAX_BLOCK]> {
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

        Some(env)
    }

    /// Orchestrates per-voice processing for a block. Returns the number of
    /// samples written to `self.scratch[..n]`. Samples beyond `written` are
    /// zeroed so the caller can mix `self.scratch[..n]` unconditionally.
    ///
    /// Layout: `prepare_block` precomputes envelope; `run_source_block` runs
    /// the per-sample DSP chain (param-mods → vib → source → filters →
    /// effects) inside its per-variant loops.
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
        let Some(env) = self.prepare_block(isr, n) else {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        };

        let written = self.run_source_block(
            &env,
            isr,
            n,
            web_pcm,
            sample_idx,
            live_input,
            input_channels,
        );
        for i in written..n {
            self.scratch[i] = [0.0; CHANNELS];
        }
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
        let Some(env) = self.prepare_block(isr, n) else {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        };

        let written = self.run_source_block(
            &env,
            isr,
            n,
            pool,
            samples,
            web_pcm,
            sample_idx,
            live_input,
            input_channels,
        );
        for i in written..n {
            self.scratch[i] = [0.0; CHANNELS];
        }
        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
