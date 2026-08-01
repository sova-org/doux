//! Voice - the core synthesis unit.

mod drums;
pub mod modulation;
mod params;
mod pluck;
mod source;

pub use modulation::{ModChain, ParamId, ParamMod};
use modulation::for_each_param;
pub use params::VoiceParams;
use pluck::PluckState;

use std::f32::consts::PI;

use crate::dsp::modal::ModalBank;
use crate::dsp::{cosf, exp2f, sinf, BrownNoise, Dahdsr, Phasor, PinkNoise, SvfMode};
use crate::effects::{
    DcBlocker, FaustChorus, FaustCoarse, FaustCrush, FaustDistort, FaustEq, FaustFlanger,
    FaustFold, FaustFreqShift, FaustHaas, FaustLadder, FaustPhaser, FaustPitchShift, FaustSmear,
    FaustSvf, FaustSvfCascade, FaustTilt, FaustVinyl, FaustWah, FaustWrap, LadderMode,
};
#[cfg(feature = "native")]
use crate::sampling::RegistrySample;
#[cfg(feature = "native")]
use crate::sampling::{GrainState, StretchState};
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
    Wah,
    Modal,
    Coarse,
    Crush,
    Fold,
    Wrap,
    Distort,
    DcBlock,
    Vinyl,
    Am,
    Rm,
    Phaser,
    Flanger,
    FreqShift,
    PitchShift,
    Eq,
    Tilt,
    Smear,
    // User arf insert — last pre-VCA stage; serial, replaces `scratch`.
    FxPatch,
    // VCA + stereo finalize — voice-core (except chorus/haas)
    Vca,
    MonoStereo,
    Chorus,
    Width,
    Haas,
    Pan,
    Trim,
}

/// Upper bound on stages a voice can emit per block. Must cover the maximal
/// chain — every push in `build_stage_program` firing at once (the nine
/// filter stages are NOT mutually exclusive), 36 stages today; allocated
/// inline on `Voice`.
pub(crate) const MAX_STAGES: usize = 40;

/// True iff `stage`'s per-sample body reads `id`, per the `stage` tags of the
/// [`for_each_param`] table. `Mirror` and `PatchLane` carry no stage tag and so
/// fall through to `false`.
fn param_in_stage(id: ParamId, stage: Stage) -> bool {
    macro_rules! stage_pairs {
        ($($v:ident => [$($kind:tt)*] $(, stage $s:ident)* ;)*) => {
            matches!((id, stage), $( $( | (ParamId::$v, Stage::$s) )* )* )
        };
    }
    for_each_param!(stage_pairs)
}

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
    pub wah: [FaustWah; CHANNELS],
    /// The modal resonator's flat bank state, one per channel. Not a Faust struct:
    /// the bank lives in `arf::modal` so the `modal` UGen and this stage run the
    /// same definition rather than two copies.
    pub modal: [[f32; arf::modal::STATE_SLOTS]; CHANNELS],
    pub coarse: [FaustCoarse; CHANNELS],
    pub crush: [FaustCrush; CHANNELS],
    pub fold_state: [FaustFold; CHANNELS],
    pub wrap_state: [FaustWrap; CHANNELS],
    pub distort_state: [FaustDistort; CHANNELS],
    pub dc_block: [DcBlocker; CHANNELS],
    pub vinyl: [FaustVinyl; CHANNELS],
    pub eq: [FaustEq; CHANNELS],
    pub tilt: [FaustTilt; CHANNELS],
    pub phaser: [FaustPhaser; CHANNELS],
    pub flanger: [FaustFlanger; CHANNELS],
    pub fshift: [FaustFreqShift; CHANNELS],
    pub pshift: [FaustPitchShift; CHANNELS],
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
            wah: std::array::from_fn(|_| FaustWah::default()),
            modal: [[0.0; arf::modal::STATE_SLOTS]; CHANNELS],
            coarse: std::array::from_fn(|_| FaustCoarse::default()),
            crush: std::array::from_fn(|_| FaustCrush::default()),
            fold_state: std::array::from_fn(|_| FaustFold::default()),
            wrap_state: std::array::from_fn(|_| FaustWrap::default()),
            distort_state: std::array::from_fn(|_| FaustDistort::default()),
            dc_block: [DcBlocker::default(); CHANNELS],
            vinyl: std::array::from_fn(|_| FaustVinyl::default()),
            eq: std::array::from_fn(|_| FaustEq::default()),
            tilt: std::array::from_fn(|_| FaustTilt::default()),
            phaser: std::array::from_fn(FaustPhaser::new),
            flanger: std::array::from_fn(FaustFlanger::new),
            fshift: std::array::from_fn(|_| FaustFreqShift::default()),
            pshift: std::array::from_fn(|_| FaustPitchShift::default()),
            smear: std::array::from_fn(|_| FaustSmear::default()),
            chorus: FaustChorus::default(),
            haas: FaustHaas::default(),
        }
    }
}

impl VoiceFxState {
    /// Re-default every field in place, clearing all cold FX state without ever
    /// building a whole `VoiceFxState` (~1.1 MB) on the stack. Note-on calls this
    /// on the audio thread, whose stack (~512 KB on a CoreAudio callback thread)
    /// a by-value rebuild would overflow in non-LTO builds. Each field's own temp
    /// stays small; `pshift` (a 512 KB delay line per channel) is cleared in place.
    pub fn reset(&mut self) {
        self.slp = std::array::from_fn(|_| FaustSvfCascade::default());
        self.shp = std::array::from_fn(|_| FaustSvfCascade::default());
        self.sbp = std::array::from_fn(|_| FaustSvfCascade::default());
        self.ladder_lp = std::array::from_fn(|_| FaustLadder::default());
        self.ladder_hp = std::array::from_fn(|_| FaustLadder::default());
        self.ladder_bp = std::array::from_fn(|_| FaustLadder::default());
        self.wah = std::array::from_fn(|_| FaustWah::default());
        self.modal = [[0.0; arf::modal::STATE_SLOTS]; CHANNELS];
        self.coarse = std::array::from_fn(|_| FaustCoarse::default());
        self.crush = std::array::from_fn(|_| FaustCrush::default());
        self.fold_state = std::array::from_fn(|_| FaustFold::default());
        self.wrap_state = std::array::from_fn(|_| FaustWrap::default());
        self.distort_state = std::array::from_fn(|_| FaustDistort::default());
        self.dc_block = [DcBlocker::default(); CHANNELS];
        self.vinyl = std::array::from_fn(|_| FaustVinyl::default());
        self.eq = std::array::from_fn(|_| FaustEq::default());
        self.tilt = std::array::from_fn(|_| FaustTilt::default());
        self.phaser = std::array::from_fn(FaustPhaser::new);
        self.flanger = std::array::from_fn(FaustFlanger::new);
        self.fshift = std::array::from_fn(|_| FaustFreqShift::default());
        for p in &mut self.pshift {
            p.reset_in_place();
        }
        self.smear = std::array::from_fn(|_| FaustSmear::default());
        self.chorus = FaustChorus::default();
        self.haas = FaustHaas::default();
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
    // op1 output history for the fmloop return; averaged like fm_fb_prev* to
    // tame the one-sample loop delay.
    pub fm_loop_prev: f32,
    pub fm_loop_prev2: f32,
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
    /// Change-detect cache for `exp2f(detune / 1200)` in
    /// [`Voice::fm_carrier_freq`]. NAN key forces the first compute.
    detune_cache_value: f32,
    detune_cache_ratio: f32,
    pub triggered: bool,
    pub time: f32,
    pub sr: f32,
    pub seed: u32,
    /// Stable identity for event addressing (`voice/N`). Survives the
    /// swap-remove in voice freeing because the whole `Voice` moves.
    pub tag: Option<usize>,
    /// SF2 exclusive class (0 = none). A new GM note in a non-zero class chokes
    /// other voices of the same class on the same orbit (hi-hat / drum groups).
    pub exclusive_class: u8,
    pub(crate) shape_active: bool,

    // === Source-specific state (one variant active per voice) ===
    pub pink_noise: [PinkNoise; CHANNELS],
    pub brown_noise: BrownNoise,
    #[cfg(not(feature = "native"))]
    pub file_source: Option<FileSource>,
    #[cfg(feature = "native")]
    pub registry_sample: Option<RegistrySample>,
    #[cfg(feature = "native")]
    pub registry_sample_b: Option<RegistrySample>,
    pub sample_blend: f32,
    /// Boxed (~40 KB phase-vocoder state) so it lives off the hot `Voice`
    /// struct — the per-voice-death `voices.swap` memcpys and the wasm shadow
    /// stack both shrink. Allocated once at `Engine::new`, reset in place.
    #[cfg(feature = "native")]
    pub stretch: Box<StretchState>,
    /// The other sample reader. ~200 bytes, so unlike `stretch` it stays inline.
    #[cfg(feature = "native")]
    pub grain: GrainState,
    pub web_sample: Option<WebSampleSource>,
    /// Live arf patch handle (`Source::Arf`): pooled Vm + control plane.
    /// Never dropped on the audio thread — see the note in [`Voice::reset`].
    pub patch: Option<crate::patch::VoicePatch>,
    /// Voice insert arf patch (`fx/<name>`), run serially over `scratch`
    /// just before the VCA. Same never-drop-on-RT contract as `patch`;
    /// cleared only by `fx/off`, New-mode reuse, or voice death.
    pub fx_patch: Option<crate::patch::VoicePatch>,
    /// Set per block when the source patch's output scrubbed a NaN/inf: its Vm
    /// state may be latched, so `gen_block` swaps in a fresh pooled Vm. Written
    /// once per block by `run_arf_block`.
    pub patch_poisoned: bool,
    /// Same, for the `fx/` insert: cleared atop `finish_block`, OR-ed per sample
    /// in `tick_fx_patch`.
    pub fx_patch_poisoned: bool,
    pub(super) drum_svf: [FaustSvf; CHANNELS],
    pub(super) drum_svf2: [FaustSvf; CHANNELS],
    /// Modal resonator bank for the metal drums (hat, cymbal). Boxed and
    /// allocated once at construction — the 16×2 filter array never lands on the
    /// audio thread by value; `reset()` zeroes it in place.
    pub(super) modal: Box<ModalBank>,
    /// Per-hit humanization for the pitched drums, drawn once at trigger:
    /// `[0]` bends pitch, `[1]` bends decay, each in `[-1, 1)`. Fixed modest depth
    /// read in the drum bodies so programmed patterns breathe.
    pub(super) drum_jitter: [f32; 2],
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
    /// the top of each `run_source_block`; executed stage-by-stage over the
    /// block by [`Voice::finish_block`]. Only the first `stage_count` entries
    /// are valid.
    pub(crate) stage_program: [Stage; MAX_STAGES],
    pub(crate) stage_count: u8,
    /// Parallel to `stage_program`: true when the stage consumes a param
    /// targeted by an active `ParamMod`, forcing that stage (and only that
    /// stage) onto the per-sample dispatch path. Only meaningful while
    /// `param_mod_count > 0` (stale otherwise, gated at the read).
    stage_modded: [bool; MAX_STAGES],
    /// Per-sample values of each active `ParamMod` for the current block,
    /// captured by `apply_mods_one` during the source loop and replayed by
    /// `restore_mods_at` for the per-sample stages. `[k][i]` = mod `k` at
    /// sample `i`; rows past `param_mod_count` and columns past the block
    /// length are stale.
    mod_traj: Box<[[f32; MAX_BLOCK]; MAX_PARAM_MODS]>,
    /// Parallel to `param_mods`: true when mod `k` held one value for every
    /// sample of the current block — a finished non-looping transition, an
    /// envelope in sustain, a sample-and-hold between draws. Cleared at block
    /// start and re-established by `apply_mods_one` from sample 0, so it is
    /// never read from a block whose source loop did not run.
    mod_constant: [bool; MAX_PARAM_MODS],
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
            fm_loop_prev: 0.0,
            fm_loop_prev2: 0.0,
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
            detune_cache_value: f32::NAN,
            detune_cache_ratio: 1.0,
            triggered: false,
            time: 0.0,
            sr,
            seed: 123456789,
            tag: None,
            exclusive_class: 0,
            shape_active: false,
            pink_noise: std::array::from_fn(|_| PinkNoise::default()),
            brown_noise: BrownNoise::default(),
            #[cfg(not(feature = "native"))]
            file_source: None,
            #[cfg(feature = "native")]
            registry_sample: None,
            #[cfg(feature = "native")]
            registry_sample_b: None,
            sample_blend: 0.0,
            #[cfg(feature = "native")]
            stretch: Box::new(StretchState::default()),
            #[cfg(feature = "native")]
            grain: GrainState::default(),
            web_sample: None,
            patch: None,
            fx_patch: None,
            patch_poisoned: false,
            fx_patch_poisoned: false,
            drum_svf: std::array::from_fn(|_| FaustSvf::default()),
            drum_svf2: std::array::from_fn(|_| FaustSvf::default()),
            modal: Box::new(ModalBank::default()),
            drum_jitter: [0.0; 2],
            pluck: Box::new(PluckState::default()),
            param_mods: [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS],
            param_mod_count: 0,
            fx: Box::new(VoiceFxState::default()),
            stage_program: [Stage::PreGain; MAX_STAGES],
            stage_count: 0,
            stage_modded: [false; MAX_STAGES],
            mod_traj: Box::new([[0.0; MAX_BLOCK]; MAX_PARAM_MODS]),
            mod_constant: [false; MAX_PARAM_MODS],
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
        self.fm_loop_prev = 0.0;
        self.fm_loop_prev2 = 0.0;
        self.fm_phase_mod = 0.0;
        self.am_lfo = Phasor::default();
        self.rm_lfo = Phasor::default();
        self.current_freq = 330.0;
        self.pink_noise = std::array::from_fn(|_| PinkNoise::default());
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
            *self.stretch = StretchState::default();
            self.grain = GrainState::default();
        }
        self.web_sample = None;
        // `patch` is deliberately untouched: dropping the arf Vm here would
        // deallocate on the audio thread. Every engine site that ends or
        // reuses a voice returns it to its pool (`PatchRegistry::retire`)
        // before calling `reset`.
        self.patch_poisoned = false;
        self.fx_patch_poisoned = false;
        // Clear all cold FX state in place — never rebuild the ~1.1 MB struct by
        // value (a stack temporary that overflows the audio thread on note-on).
        self.fx.reset();
        self.param_mods = [(ParamId::Gain, ParamMod::default()); MAX_PARAM_MODS];
        self.param_mod_count = 0;
        self.triggered = false;
        self.time = 0.0;
        self.tag = None;
        self.exclusive_class = 0;
        *self.scratch = [[0.0; CHANNELS]; MAX_BLOCK];
        self.nch = 1;
        self.spread_cache_value = f32::NAN;
        self.spread_detune_ratios = [1.0; 3];
        self.shape_active = false;
        self.sr = 44100.0;
        self.seed = 123456789;
        self.drum_svf = std::array::from_fn(|_| FaustSvf::default());
        self.drum_svf2 = std::array::from_fn(|_| FaustSvf::default());
        self.modal.reset_in_place();
        self.drum_jitter = [0.0; 2];
        // O(1): the 32 KB delay line is re-zeroed lazily by `run_pluck` on the
        // first sample of a pluck note, never here on every note-on.
        self.pluck.primed = false;
        self.stage_count = 0;
    }

    /// No-op: effects are pre-allocated at init.
    pub fn ensure_effects(&mut self) {}

    #[inline]
    pub(super) fn rand(&mut self) -> f32 {
        // xorshift32; guard the 0 fixed point (the rolling voice_seed can reach 0
        // once in 2^32, and 0 is a fixed point of xorshift).
        let mut x = if self.seed == 0 { 0x9E3779B9 } else { self.seed };
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x >> 8) as f32 / 16_777_216.0 // 24-bit, [0,1)
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
        macro_rules! read_field {
            (plain $f:ident, $recv:expr) => {
                $recv.$f
            };
            (opt $f:ident = $d:expr, $recv:expr) => {
                $recv.$f.unwrap_or($d)
            };
        }
        macro_rules! read_arms {
            ($($v:ident => [$($kind:tt)*] $(, stage $s:ident)* ;)*) => {
                match id {
                    $(ParamId::$v => read_field!($($kind)* , self.params),)*
                    ParamId::Mirror => self.params.shape.mirror,
                    // The current lane value: the declared default until something writes
                    // it (VoicePatch::new fills defaults), 0.0 with no source patch.
                    ParamId::PatchLane(lane) => self
                        .patch
                        .as_ref()
                        .map_or(0.0, |p| p.control[lane as usize]),
                }
            };
        }
        for_each_param!(read_arms)
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

    /// True if any active `ParamMod` targets a param `stage` consumes, per the
    /// `param_in_stage` table.
    fn stage_uses_modded_param(&self, stage: Stage) -> bool {
        (0..self.param_mod_count as usize)
            .any(|k| param_in_stage(self.param_mods[k].0, stage))
    }

    /// True iff every active mod `stage` reads held one value across the whole
    /// block. Such a stage can take the block path even though it is flagged in
    /// `stage_modded`: with `mod_traj[k][i] == mod_traj[k][n-1]` for every `i`,
    /// `restore_mods_at(i)` writes the same params on every sample, so the
    /// per-sample replay and the block dispatch run the same operation sequence
    /// on the same values. A mod that is still moving keeps its stage per-sample.
    fn stage_mods_constant(&self, stage: Stage) -> bool {
        (0..self.param_mod_count as usize)
            .all(|k| self.mod_constant[k] || !param_in_stage(self.param_mods[k].0, stage))
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
        if self.params.wah > 0.0 {
            push!(Stage::Wah);
        }
        if self.params.modal > 0.0 || self.mod_targets(ParamId::Modal) {
            push!(Stage::Modal);
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
        if self.params.vinyl > 0.0 {
            push!(Stage::Vinyl);
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
        if self.params.fshift != 0.0 || self.mod_targets(ParamId::Fshift) {
            push!(Stage::FreqShift);
        }
        if self.params.pshift != 0.0 || self.mod_targets(ParamId::Pshift) {
            push!(Stage::PitchShift);
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
        if self.fx_patch.is_some() {
            push!(Stage::FxPatch);
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

        // Per-stage dispatch flags: a stage touching a modulated param runs
        // per-sample in `finish_block`; everything else stays block-rate.
        if self.param_mod_count > 0 {
            for k in 0..count as usize {
                self.stage_modded[k] = self.stage_uses_modded_param(self.stage_program[k]);
            }
            // `apply_mods_one` re-establishes these from sample 0 of this block.
            // Clearing here means a block whose source loop never ran cannot
            // demote a stage on the previous block's verdict.
            self.mod_constant = [false; MAX_PARAM_MODS];
        }
    }

    /// Rewind every active mod target to its captured value at sample `i`,
    /// so a per-sample stage sees exactly the params `apply_mods_one` wrote
    /// while the source loop was at sample `i`.
    #[inline]
    fn restore_mods_at(&mut self, i: usize) {
        for k in 0..self.param_mod_count as usize {
            let id = self.param_mods[k].0;
            let val = self.mod_traj[k][i];
            self.write_param(id, val);
        }
    }

    /// Stage executor: runs each stage in `stage_program[..stage_count]` once
    /// over `scratch[..n]`. Called from `run_source_block` after the source
    /// body has filled `scratch[..n]`.
    ///
    /// Stages whose params are all unmodulated dispatch through the per-stage
    /// `process_block` APIs — params read once at block entry, coefficient
    /// recompute amortized, inner loop vectorizable. A stage flagged in
    /// `stage_modded` replays the per-sample trajectory captured by
    /// `apply_mods_one` and dispatches through [`Voice::tick_stage`], so
    /// per-sample modulation is preserved exactly. Stage-by-stage over the
    /// block is mathematically equivalent to sample-by-sample over stages:
    /// each stage reads only `scratch[i]` and its own state (filter state
    /// machines are identical; only loop order changes).
    /// One sample of the user arf insert (`fx/<name>`): serial over
    /// `scratch[i]`, pre-VCA. Width-preserving — `finish_block` snapshots
    /// `nch` once, so the insert must never change the voice's width: a
    /// stereo patch on a mono voice is downmixed, and `scratch[i][1]` (the
    /// spread side signal on mono voices) is left untouched. No 0.7
    /// headroom: an identity `{ in out }` insert must be unity. An effect's
    /// control plane carries only the transport lane (patch.rs contract),
    /// latched per chunk; the non-finite scrub matches `run_arf_block`.
    #[inline]
    fn tick_fx_patch(&mut self, i: usize) {
        let nch = self.nch;
        if let Some(p) = self.fx_patch.as_mut() {
            let program = p.entry.program();
            let in_ch = program.in_channels();
            let width = program.audio_channels().min(CHANNELS);
            let l = self.scratch[i][0];
            let r = self.scratch[i][if nch == 2 { 1 } else { 0 }];
            let input = if in_ch == 2 {
                [l, r]
            } else if nch == 2 {
                [(l + r) * 0.5, 0.0]
            } else {
                [l, 0.0]
            };
            let mut out = [0.0f32; CHANNELS];
            p.vm.tick_frame(
                program,
                p.frame_pos,
                &input[..in_ch],
                &p.control[..program.control_len()],
                &mut out[..width],
            );
            p.frame_pos += 1;
            self.fx_patch_poisoned |= crate::patch::scrub_non_finite(&mut out);
            let (w0, w1) = (out[0], out[1]);
            if nch == 2 {
                self.scratch[i][0] = w0;
                self.scratch[i][1] = if width == 2 { w1 } else { w0 };
            } else {
                self.scratch[i][0] = if width == 2 { (w0 + w1) * 0.5 } else { w0 };
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub(crate) fn finish_block(&mut self, env: &[f32], n: usize, isr: f32) {
        let sr = self.sr;
        let nch = self.nch;
        // Re-accumulated per block by `tick_fx_patch` (OR-ed per sample).
        self.fx_patch_poisoned = false;
        let has_mods = self.param_mod_count > 0;
        for k in 0..self.stage_count as usize {
            let stage = self.stage_program[k];
            // A flagged stage whose every mod went constant this block is
            // demoted back to the block path: the per-sample replay would write
            // the same params on all `n` samples, so the two paths agree.
            if has_mods && self.stage_modded[k] && !self.stage_mods_constant(stage) {
                for i in 0..n {
                    self.restore_mods_at(i);
                    self.tick_stage(stage, i, env[i], sr, isr);
                }
            } else {
                self.tick_stage_block(stage, env, n, sr, isr, nch);
            }
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
            Stage::Wah => {
                if self.params.wah > 0.0 {
                    let amount = self.params.wah;
                    let peak = self.params.wahpeak;
                    let sens = self.params.wahsens;
                    let manual = self.params.wahmanual;
                    for c in 0..nch {
                        self.fx.wah[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            amount,
                            peak,
                            sens,
                            manual,
                            sr,
                        );
                    }
                }
            }
            Stage::Modal => {
                if self.params.modal > 0.0 {
                    // The gate above already excludes 0, negatives and NaN, so only the
                    // top needs bounding: past 1 the `x + (y - x) * mix` dry term inverts.
                    let mix = self.params.modal.min(1.0);
                    let freq = self.params.modalfreq;
                    let decay = self.params.modaldecay;
                    let structure = self.params.modalstruct;
                    let bright = self.params.modalbright;
                    for c in 0..nch {
                        let state = &mut self.fx.modal[c];
                        for s in 0..n {
                            let x = self.scratch[s][c];
                            let y =
                                arf::modal::tick(state, x, freq, decay, structure, bright, sr);
                            self.scratch[s][c] = x + (y - x) * mix;
                        }
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
                    let asym = self.params.distortasym;
                    for c in 0..nch {
                        self.fx.distort_state[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            amount,
                            postgain,
                            mode,
                            asym,
                        );
                    }
                }
            }
            Stage::DcBlock => {
                for c in 0..nch {
                    self.fx.dc_block[c].process_block(&mut self.scratch[..n], n, c);
                }
            }
            Stage::Vinyl => {
                if self.params.vinyl > 0.0 {
                    let amount = self.params.vinyl;
                    let wow = self.params.vinylwow;
                    let noise = self.params.vinylnoise;
                    let tone = self.params.vinyltone;
                    let kind = self.params.vinyltype.to_index();
                    for c in 0..nch {
                        self.fx.vinyl[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            amount,
                            wow,
                            noise,
                            tone,
                            kind,
                            sr,
                        );
                    }
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
                    let mode = self.params.flangermode.to_index();
                    for c in 0..nch {
                        self.fx.flanger[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            rate,
                            depth,
                            fb,
                            mode,
                            sr,
                        );
                    }
                }
            }
            Stage::FreqShift => {
                if self.params.fshift != 0.0 {
                    let shift = self.params.fshift;
                    for c in 0..nch {
                        self.fx.fshift[c].process_block(&mut self.scratch[..n], n, c, shift, sr);
                    }
                }
            }
            Stage::PitchShift => {
                if self.params.pshift != 0.0 {
                    let shift = self.params.pshift;
                    let window = self.params.pshiftwin;
                    for c in 0..nch {
                        self.fx.pshift[c].process_block(
                            &mut self.scratch[..n],
                            n,
                            c,
                            shift,
                            window,
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
            Stage::FxPatch => {
                for i in 0..n {
                    self.tick_fx_patch(i);
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
                // Applied unconditionally: skipping it at pan == 0.5 left centre
                // 3 dB above every other position, a cliff between two adjacent
                // float values that a modulated pan crosses as a step.
                let pan_pos = self.params.pan * PI / 2.0;
                let l = cosf(pan_pos);
                let r = sinf(pan_pos);
                for i in 0..n {
                    self.scratch[i][0] *= l;
                    self.scratch[i][1] *= r;
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
            Stage::Wah => {
                if self.params.wah > 0.0 {
                    let amount = self.params.wah;
                    let peak = self.params.wahpeak;
                    let sens = self.params.wahsens;
                    let manual = self.params.wahmanual;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.wah[c].process(x, amount, peak, sens, manual, sr);
                    }
                }
            }
            Stage::Modal => {
                if self.params.modal > 0.0 {
                    // The gate above already excludes 0, negatives and NaN, so only the
                    // top needs bounding: past 1 the `x + (y - x) * mix` dry term inverts.
                    let mix = self.params.modal.min(1.0);
                    let freq = self.params.modalfreq;
                    let decay = self.params.modaldecay;
                    let structure = self.params.modalstruct;
                    let bright = self.params.modalbright;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        let y = arf::modal::tick(
                            &mut self.fx.modal[c],
                            x,
                            freq,
                            decay,
                            structure,
                            bright,
                            sr,
                        );
                        self.scratch[i][c] = x + (y - x) * mix;
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
                    let asym = self.params.distortasym;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.distort_state[c].process(x, dist_amount, postgain, mode, asym);
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
            Stage::Vinyl => {
                if self.params.vinyl > 0.0 {
                    let amount = self.params.vinyl;
                    let wow = self.params.vinylwow;
                    let noise = self.params.vinylnoise;
                    let tone = self.params.vinyltone;
                    let kind = self.params.vinyltype.to_index();
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.vinyl[c].process(x, amount, wow, noise, tone, kind, sr);
                    }
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
                    let mode = self.params.flangermode.to_index();
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] =
                            self.fx.flanger[c].process(x, rate, depth, fb, mode, sr);
                    }
                }
            }
            Stage::FreqShift => {
                if self.params.fshift != 0.0 {
                    let shift = self.params.fshift;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.fshift[c].process(x, shift, sr);
                    }
                }
            }
            Stage::PitchShift => {
                if self.params.pshift != 0.0 {
                    let shift = self.params.pshift;
                    let window = self.params.pshiftwin;
                    for c in 0..nch {
                        let x = self.scratch[i][c];
                        self.scratch[i][c] = self.fx.pshift[c].process(x, shift, window, sr);
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
            // Unreachable through the modded dispatch (FxPatch consumes no
            // modulatable param), but kept real for exhaustiveness.
            Stage::FxPatch => self.tick_fx_patch(i),
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
                let pan_pos = self.params.pan * PI / 2.0;
                let l = cosf(pan_pos);
                let r = sinf(pan_pos);
                self.scratch[i][0] *= l;
                self.scratch[i][1] *= r;
            }
            Stage::Trim => {
                for c in 0..CHANNELS {
                    self.scratch[i][c] *= VOICE_OUTPUT_TRIM;
                }
            }
        }
    }

    /// Sample-rate param-mod application: ticks each `ParamMod` once, writes
    /// the result to its target, and captures it in `mod_traj[..][i]` for the
    /// per-sample stages in `finish_block`. Called once per sample inside the
    /// source loop; `i` is the sample index within the block.
    #[inline]
    fn apply_mods_one(&mut self, isr: f32, i: usize) {
        for k in 0..self.param_mod_count as usize {
            let (id, ref mut m) = self.param_mods[k];
            let val = m.tick(isr);
            self.mod_traj[k][i] = val;
            // Constancy over the block, for the stage demotion in `finish_block`.
            // Sample 0 seeds it (the comparison is against the value just
            // stored, so it holds trivially); later samples clear it on the
            // first departure. Exact equality is the point — a value that only
            // nearly repeats must stay on the per-sample path, and a NaN (which
            // compares unequal to itself) falls back to it too.
            self.mod_constant[k] = (i == 0 || self.mod_constant[k]) && val == self.mod_traj[k][0];
            self.write_param(id, val);
        }
    }

    /// Per-sample voice "pre-source" tick: apply param-mods, tick the FM
    /// modulator phasors at the pre-vib carrier, advance the vib LFO. Returns
    /// the post-vib carrier freq for the source body to use. `i` is the
    /// sample index within the block (for the mod-trajectory capture).
    #[inline]
    pub(crate) fn tick_pre(&mut self, isr: f32, i: usize) -> f32 {
        if self.param_mod_count > 0 {
            self.apply_mods_one(isr, i);
        }
        let pre_vib = self.fm_carrier_freq();
        self.tick_fm_pm(pre_vib, isr);
        self.compute_freq_one(pre_vib, isr)
    }

    fn write_param(&mut self, id: ParamId, val: f32) {
        macro_rules! write_field {
            (plain $f:ident, $recv:expr, $val:expr) => {
                $recv.$f = $val
            };
            (opt $f:ident = $d:expr, $recv:expr, $val:expr) => {
                $recv.$f = Some($val)
            };
        }
        macro_rules! write_arms {
            ($($v:ident => [$($kind:tt)*] $(, stage $s:ident)* ;)*) => {
                match id {
                    $(ParamId::$v => write_field!($($kind)* , self.params, val),)*
                    ParamId::Mirror => {
                        self.params.shape.mirror = val;
                        self.shape_active = self.params.shape.is_active();
                    }
                    ParamId::PatchLane(lane) => {
                        if let Some(p) = self.patch.as_mut() {
                            p.control[lane as usize] = val;
                        }
                    }
                }
            };
        }
        for_each_param!(write_arms)
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
    pub(crate) fn fm_carrier_freq(&mut self) -> f32 {
        let mut f = self.params.freq;
        let detune = self.params.detune;
        if detune != 0.0 {
            // Change-detect cache (same idiom as `spread_detune_ratios`):
            // `exp2f` runs once per detune value instead of once per sample.
            // A pure-function cache keyed on the exact input — never stale.
            if detune != self.detune_cache_value {
                self.detune_cache_value = detune;
                self.detune_cache_ratio = exp2f(detune / 1200.0);
            }
            f *= self.detune_cache_ratio;
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
    ///
    /// `fmloop` adds the return leg op1 → op2, closing the two-op pair into a
    /// feedback loop; only active when fm2 > 0.
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

                // op1 → op2 return, closing the loop; averaged like fb_turns.
                let loop_turns =
                    (self.fm_loop_prev + self.fm_loop_prev2) * 0.5 * self.params.fmloop * INV_TAU;

                let mod2_freq = freq_pre_vib * self.params.fm2h;
                let mod2 = self
                    .fm2_phasor
                    .lfo_pm(shape, mod2_freq, isr, fb_turns + loop_turns);
                self.fm_fb_prev2 = self.fm_fb_prev;
                self.fm_fb_prev = mod2;

                let mod1_freq = freq_pre_vib * self.params.fmh;
                let mod1 = self
                    .fm_phasor
                    .lfo_pm(shape, mod1_freq, isr, a * mod2 * INV_TAU);
                self.fm_loop_prev2 = self.fm_loop_prev;
                self.fm_loop_prev = mod1;

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
        // Per-hit variation for the drums: `reset` pins the spread phasors at
        // `i/7`, which clones the partial stack on every hit. Randomizing them
        // (the seed advances per hit) de-correlates the partials so repeated hits
        // vary instead of phasing identically. The two jitter values then bend
        // pitch and decay per hit (read in the pitched drum bodies). The main
        // `phasor` stays at 0 — kick/tom/rim want a deterministic body attack.
        if self.params.sound.drum_defaults().is_some() {
            for i in 0..self.spread_phasors.len() {
                self.spread_phasors[i].phase = self.rand();
            }
            self.drum_jitter = [self.rand() * 2.0 - 1.0, self.rand() * 2.0 - 1.0];
        }
        for i in 0..self.param_mod_count as usize {
            self.param_mods[i].1.trigger(self.params.gate);
        }
    }

    /// Block-rate preamble: trigger the envelope (once) and precompute `n`
    /// envelope samples into the caller's `env` buffer (a stack array in the
    /// `process_block` variants — passing it by `&mut` avoids the 1 KB by-value
    /// return copy). Param-mods and carrier freq are computed per-sample inside
    /// the source loop, so they aren't touched here. Returns `false` (env
    /// untouched-past-`n` / meaningless) if the envelope is `Off` after the block.
    pub(crate) fn prepare_block(&mut self, isr: f32, n: usize, env: &mut [f32; MAX_BLOCK]) -> bool {
        if !self.triggered {
            self.trigger_envelopes();
            self.triggered = true;
        }

        self.dahdsr.update_block(
            env,
            n,
            isr,
            self.params.envdelay,
            self.params.attack,
            self.params.hold,
            self.params.decay,
            self.params.sustain,
            self.params.release,
        );
        !self.dahdsr.is_off()
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
        let mut env = [0.0f32; MAX_BLOCK];
        if !self.prepare_block(isr, n, &mut env) {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        }

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
        let mut env = [0.0f32; MAX_BLOCK];
        if !self.prepare_block(isr, n, &mut env) {
            for i in 0..n {
                self.scratch[i] = [0.0; CHANNELS];
            }
            return 0;
        }

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
