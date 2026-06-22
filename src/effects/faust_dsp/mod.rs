//! Faust-generated DSP effects (experimental).
//!
//! `crush` (bit-depth reduction) and `fold` (triangle wavefolder) are compiled
//! from `dsp/*.dsp` by the Faust compiler (`faust -lang rust`) and committed as
//! `*_gen.rs` beside this file. Regenerate with `dsp/regen.sh`. The generated
//! code depends only on the `faust-types` trait crate at runtime.
//!
//! Both effects are sample-rate-independent: their `instance_constants` only
//! stores the rate and no coefficient derives from it, so the wrappers init the
//! Faust instance at a fixed nominal rate and produce identical output at any
//! actual rate.
//!
//! Each generated module is mono (1-in/1-out) with the effect amount at slider
//! index 0. The wrappers mirror the `process` / `process_block` convention of
//! the hand-written voice-insert effects so call sites are unchanged.

use super::reverb::ReverbParams;
use super::LadderMode;
use crate::dsp::SvfMode;
use crate::types::{StereoFrame, MAX_BLOCK};
use faust_types::{FaustDsp, ParamIndex, UI};

/// Nominal init rate for the sample-rate-independent Faust effects (Hz).
const NOMINAL_SR: i32 = 48_000;

/// Effect amount lives at slider index 0 in the single-parameter generated DSPs.
const AMOUNT: ParamIndex = ParamIndex(0);

/// Read-only [`UI`] visitor that recovers the slider index Faust assigned to a
/// label. Faust sorts sliders alphabetically and assigns `ParamIndex` in that
/// order; the wrappers below hold hand-written `ParamIndex` constants matching
/// that order. [`slider_index`] + [`assert_slider_idx`] let each constructor
/// `debug_assert` the two agree, so a future `.dsp` edit that shifts the slider
/// order panics loudly in debug instead of silently routing a param into the
/// wrong DSP input.
struct IdxProbe {
    want: &'static str,
    found: Option<i32>,
}

impl UI<f32> for IdxProbe {
    fn add_horizontal_slider(
        &mut self,
        label: &str,
        param: ParamIndex,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) {
        if label == self.want {
            self.found = Some(param.0);
        }
    }
    // The generated insert/filter DSPs only ever emit horizontal sliders; every
    // other `UI` widget is a no-op for index probing.
    fn open_tab_box(&mut self, _: &str) {}
    fn open_horizontal_box(&mut self, _: &str) {}
    fn open_vertical_box(&mut self, _: &str) {}
    fn close_box(&mut self) {}
    fn add_button(&mut self, _: &str, _: ParamIndex) {}
    fn add_check_button(&mut self, _: &str, _: ParamIndex) {}
    fn add_vertical_slider(&mut self, _: &str, _: ParamIndex, _: f32, _: f32, _: f32, _: f32) {}
    fn add_num_entry(&mut self, _: &str, _: ParamIndex, _: f32, _: f32, _: f32, _: f32) {}
    fn add_horizontal_bargraph(&mut self, _: &str, _: ParamIndex, _: f32, _: f32) {}
    fn add_vertical_bargraph(&mut self, _: &str, _: ParamIndex, _: f32, _: f32) {}
    fn declare(&mut self, _: Option<ParamIndex>, _: &str, _: &str) {}
}

/// The slider index Faust assigned to `label` in DSP `D`, or `None` if absent.
fn slider_index<D: FaustDsp<T = f32>>(label: &'static str) -> Option<i32> {
    let mut probe = IdxProbe {
        want: label,
        found: None,
    };
    D::build_user_interface_static(&mut probe);
    probe.found
}

/// Debug-assert each `label => ParamIndex` claim against the DSP's actual slider
/// order. Compiled out in release.
macro_rules! assert_slider_idx {
    ($dsp:ty, $($label:literal => $idx:expr),+ $(,)?) => {{
        $(
            debug_assert_eq!(
                slider_index::<$dsp>($label),
                Some($idx),
                "Faust slider `{}` is not at ParamIndex({}) — a .dsp regen drifted from the wrapper",
                $label,
                $idx,
            );
        )+
    }};
}

// One self-contained module per generated DSP: each file defines its own
// `FaustFloat`, `ffi` helpers and `FAUST_*` consts, so they cannot share a
// module. `use faust_types::*` supplies the trait/UI/Meta/ParamIndex symbols
// the generated code references unqualified.
mod crush_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("crush_gen.rs");
}
mod fold_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("fold_gen.rs");
}
mod svf_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("svf_gen.rs");
}
mod coarse_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("coarse_gen.rs");
}
mod wrap_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("wrap_gen.rs");
}
mod distort_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("distort_gen.rs");
}
mod tilt_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("tilt_gen.rs");
}
mod eq_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("eq_gen.rs");
}
mod phaser_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("phaser_gen.rs");
}
mod chorus_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("chorus_gen.rs");
}
mod flanger_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("flanger_gen.rs");
}
mod smear_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("smear_gen.rs");
}
mod haas_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("haas_gen.rs");
}
mod ladder_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("ladder_gen.rs");
}
mod svf24_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("svf24_gen.rs");
}
mod vital_rev_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("vital_rev_gen.rs");
}
mod jpverb_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("jpverb_gen.rs");
}

/// Run a mono Faust DSP for one sample. Used by the per-sample dispatch path so
/// a modulated param threads in at sample rate. Params must be set beforehand.
#[inline]
fn run_one<D: FaustDsp<T = f32>>(dsp: &mut D, x: f32) -> f32 {
    let input = [x];
    let mut output = [0.0f32];
    dsp.compute(1, &[&input[..]], &mut [&mut output[..]]);
    output[0]
}

/// Run a mono Faust DSP over `n` samples of channel `ch` in `buf`, in place.
/// Params (held for the block) must be set beforehand.
#[inline]
fn run_block<D: FaustDsp<T = f32>>(dsp: &mut D, buf: &mut [StereoFrame], n: usize, ch: usize) {
    let mut input = [0.0f32; MAX_BLOCK];
    let mut output = [0.0f32; MAX_BLOCK];
    for i in 0..n {
        input[i] = buf[i][ch];
    }
    dsp.compute(n as i32, &[&input[..n]], &mut [&mut output[..n]]);
    for i in 0..n {
        buf[i][ch] = output[i];
    }
}

/// Run a stereo (2-in/2-out) Faust DSP for one frame. Params set beforehand.
#[inline]
fn run_one_stereo<D: FaustDsp<T = f32>>(dsp: &mut D, l: f32, r: f32) -> [f32; 2] {
    let in_l = [l];
    let in_r = [r];
    let mut out_l = [0.0f32];
    let mut out_r = [0.0f32];
    dsp.compute(1, &[&in_l[..], &in_r[..]], &mut [&mut out_l[..], &mut out_r[..]]);
    [out_l[0], out_r[0]]
}

/// Run a stereo (2-in/2-out) Faust DSP over `n` frames of `buf`, in place.
/// Params (held for the block) set beforehand.
#[inline]
fn run_block_stereo<D: FaustDsp<T = f32>>(dsp: &mut D, buf: &mut [StereoFrame], n: usize) {
    let mut in_l = [0.0f32; MAX_BLOCK];
    let mut in_r = [0.0f32; MAX_BLOCK];
    let mut out_l = [0.0f32; MAX_BLOCK];
    let mut out_r = [0.0f32; MAX_BLOCK];
    for i in 0..n {
        in_l[i] = buf[i][0];
        in_r[i] = buf[i][1];
    }
    dsp.compute(
        n as i32,
        &[&in_l[..n], &in_r[..n]],
        &mut [&mut out_l[..n], &mut out_r[..n]],
    );
    for i in 0..n {
        buf[i][0] = out_l[i];
        buf[i][1] = out_r[i];
    }
}

/// Allocate a Faust DSP directly on the heap, zeroed, without ever materializing
/// it on the stack. The reverb DSPs carry multi-megabyte inline delay arrays, so
/// by-value construction (`Box::new(D::new())`) overflows the stack in debug.
/// Faust's own `new()` only zeroes every field; the subsequent `init()` writes
/// the sample-rate constants and clears state — so a zeroed heap struct + `init`
/// is equivalent.
#[inline]
fn boxed_zeroed<D>() -> Box<D> {
    let mut b = Box::<D>::new_uninit();
    // SAFETY: every generated Faust DSP struct is `#[repr(C)]` and holds only
    // `i32`/`f32` scalars and arrays, for which the all-zero bit pattern is the
    // valid initial value Faust's `new()` writes; `init()` is called next.
    unsafe {
        std::ptr::write_bytes(b.as_mut_ptr(), 0, 1);
        b.assume_init()
    }
}

/// Wrap a mono Faust DSP as a voice-insert effect matching the hand-written
/// `process` / `process_block(buf, n, ch, amount)` convention, so call sites
/// are unchanged.
macro_rules! faust_insert {
    ($wrapper:ident, $dsp:path, $label:literal, $doc:literal) => {
        #[doc = $doc]
        pub struct $wrapper {
            dsp: $dsp,
        }

        impl Default for $wrapper {
            fn default() -> Self {
                let mut dsp = <$dsp>::new();
                dsp.init(NOMINAL_SR);
                debug_assert_eq!(
                    slider_index::<$dsp>($label),
                    Some(AMOUNT.0),
                    "Faust slider `{}` is not at ParamIndex(0) — a .dsp regen drifted from the wrapper",
                    $label,
                );
                Self { dsp }
            }
        }

        impl $wrapper {
            /// Process one sample. Used by the per-sample dispatch path so a
            /// modulated `amount` threads in at sample rate.
            #[inline]
            pub fn process(&mut self, x: f32, amount: f32) -> f32 {
                self.dsp.set_param(AMOUNT, amount);
                run_one(&mut self.dsp, x)
            }

            /// Process `n` samples of channel `ch` in place. `amount` is held
            /// for the whole block (block-rate param), matching Faust's model.
            #[inline]
            pub fn process_block(
                &mut self,
                buf: &mut [StereoFrame],
                n: usize,
                ch: usize,
                amount: f32,
            ) {
                self.dsp.set_param(AMOUNT, amount);
                run_block(&mut self.dsp, buf, n, ch);
            }
        }
    };
}

faust_insert!(
    FaustCrush,
    crush_dsp::CrushDsp,
    "crush",
    "Bit-depth reduction (bitcrusher), Faust-generated. `amount` = target bit depth."
);
faust_insert!(
    FaustCoarse,
    coarse_dsp::CoarseDsp,
    "coarse",
    "Sample-rate reduction (decimation), Faust-generated. `amount` = hold factor."
);
faust_insert!(
    FaustWrap,
    wrap_dsp::WrapDsp,
    "wrap",
    "Phase-wrap distortion, Faust-generated. `amount` in [0, 10]."
);

/// State-variable filter (lp/hp/bp), Faust-generated. Public `cutoff` field set
/// by callers, `process` / `process_block(.., mode, q, sr)` — the interface the
/// voice (and drum) dispatch call sites expect, so they are unchanged. Unlike the memoryless effects this DSP is sample-rate
/// dependent, so it (re)initializes the Faust instance the first time a sample
/// rate arrives (and if it ever changes).
pub struct FaustSvf {
    /// Cutoff in Hz, set by the caller before each `process` / `process_block`.
    pub cutoff: f32,
    dsp: svf_dsp::SvfDsp,
    /// Sample rate the Faust instance was initialized at (0.0 = not yet).
    sr: f32,
}

impl FaustSvf {
    // Slider order in svf.dsp (the a_/b_/c_ prefixes force alphabetical to equal
    // this order): cutoff, q, mode.
    const CUTOFF: ParamIndex = ParamIndex(0);
    const Q: ParamIndex = ParamIndex(1);
    const MODE: ParamIndex = ParamIndex(2);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, mode: SvfMode, q: f32) {
        let mode_idx = match mode {
            SvfMode::Lp => 0.0,
            SvfMode::Hp => 1.0,
            SvfMode::Bp => 2.0,
        };
        self.dsp.set_param(Self::CUTOFF, self.cutoff);
        self.dsp.set_param(Self::MODE, mode_idx);
        self.dsp.set_param(Self::Q, q);
    }

    #[inline]
    pub fn process(&mut self, x: f32, mode: SvfMode, q: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(mode, q);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        mode: SvfMode,
        q: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(mode, q);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustSvf {
    fn default() -> Self {
        assert_slider_idx!(svf_dsp::SvfDsp,
            "a_cutoff" => Self::CUTOFF.0, "b_q" => Self::Q.0, "c_mode" => Self::MODE.0);
        Self {
            cutoff: 0.0,
            dsp: svf_dsp::SvfDsp::new(),
            sr: 0.0,
        }
    }
}

/// Soft-saturation distortion, Faust-generated. Two params — drive (`distort`)
/// and output gain (`distortvol`) — so it needs its own wrapper rather than the
/// single-parameter `faust_insert!`. Memoryless and sample-rate independent, so
/// the instance initializes at a fixed nominal rate.
pub struct FaustDistort {
    dsp: distort_dsp::DistortDsp,
}

impl FaustDistort {
    const DISTORT: ParamIndex = ParamIndex(0);
    const DISTORTVOL: ParamIndex = ParamIndex(1);
    const DISTORTMODE: ParamIndex = ParamIndex(2);

    #[inline]
    fn write_params(&mut self, amount: f32, postgain: f32, mode: f32) {
        self.dsp.set_param(Self::DISTORT, amount);
        self.dsp.set_param(Self::DISTORTVOL, postgain);
        self.dsp.set_param(Self::DISTORTMODE, mode);
    }

    #[inline]
    pub fn process(&mut self, x: f32, amount: f32, postgain: f32, mode: f32) -> f32 {
        self.write_params(amount, postgain, mode);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        postgain: f32,
        mode: f32,
    ) {
        self.write_params(amount, postgain, mode);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustDistort {
    fn default() -> Self {
        assert_slider_idx!(distort_dsp::DistortDsp,
            "a_distort" => Self::DISTORT.0, "b_distortvol" => Self::DISTORTVOL.0,
            "c_distortmode" => Self::DISTORTMODE.0);
        let mut dsp = distort_dsp::DistortDsp::new();
        dsp.init(NOMINAL_SR);
        Self { dsp }
    }
}

/// Reflective wavefolder with a selectable shape, Faust-generated. Two params —
/// fold `amount` and `mode` (0=triangle, 1=sine, 2=wrap) — so it needs its own
/// wrapper rather than the single-parameter `faust_insert!`. Memoryless and
/// sample-rate independent, so the instance initializes at a fixed nominal rate.
pub struct FaustFold {
    dsp: fold_dsp::FoldDsp,
}

impl FaustFold {
    const FOLD: ParamIndex = ParamIndex(0);
    const FOLDMODE: ParamIndex = ParamIndex(1);

    #[inline]
    fn write_params(&mut self, amount: f32, mode: f32) {
        self.dsp.set_param(Self::FOLD, amount);
        self.dsp.set_param(Self::FOLDMODE, mode);
    }

    #[inline]
    pub fn process(&mut self, x: f32, amount: f32, mode: f32) -> f32 {
        self.write_params(amount, mode);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        mode: f32,
    ) {
        self.write_params(amount, mode);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustFold {
    fn default() -> Self {
        assert_slider_idx!(fold_dsp::FoldDsp,
            "a_fold" => Self::FOLD.0, "b_foldmode" => Self::FOLDMODE.0);
        let mut dsp = fold_dsp::FoldDsp::new();
        dsp.init(NOMINAL_SR);
        Self { dsp }
    }
}

/// Single-knob tilt EQ (high shelf), Faust-generated. Mirrors the hand-written
/// `Tilt` interface (`process(x, tilt, sr)`). Sample-rate dependent, so it
/// lazily (re)initializes the Faust instance when `sr` first arrives.
pub struct FaustTilt {
    dsp: tilt_dsp::TiltDsp,
    sr: f32,
}

impl FaustTilt {
    const TILT: ParamIndex = ParamIndex(0);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32, tilt: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::TILT, tilt);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        tilt: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::TILT, tilt);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustTilt {
    fn default() -> Self {
        assert_slider_idx!(tilt_dsp::TiltDsp, "tilt" => Self::TILT.0);
        Self {
            dsp: tilt_dsp::TiltDsp::new(),
            sr: 0.0,
        }
    }
}

/// 3-band EQ (low shelf / mid peak / high shelf), Faust-generated. Mirrors the
/// hand-written `Eq` interface; arg order matches it exactly so the voice
/// dispatch call sites are unchanged. Sample-rate dependent (lazy re-init).
pub struct FaustEq {
    dsp: eq_dsp::EqDsp,
    sr: f32,
}

impl FaustEq {
    const LO_DB: ParamIndex = ParamIndex(0);
    const LO_F: ParamIndex = ParamIndex(1);
    const MID_DB: ParamIndex = ParamIndex(2);
    const MID_F: ParamIndex = ParamIndex(3);
    const HI_DB: ParamIndex = ParamIndex(4);
    const HI_F: ParamIndex = ParamIndex(5);
    const MID_Q: ParamIndex = ParamIndex(6);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn write_params(
        &mut self,
        lo_db: f32,
        mid_db: f32,
        hi_db: f32,
        lo_f: f32,
        mid_f: f32,
        hi_f: f32,
        mid_q: f32,
    ) {
        self.dsp.set_param(Self::LO_DB, lo_db);
        self.dsp.set_param(Self::LO_F, lo_f);
        self.dsp.set_param(Self::MID_DB, mid_db);
        self.dsp.set_param(Self::MID_F, mid_f);
        self.dsp.set_param(Self::HI_DB, hi_db);
        self.dsp.set_param(Self::HI_F, hi_f);
        self.dsp.set_param(Self::MID_Q, mid_q);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        x: f32,
        lo_db: f32,
        mid_db: f32,
        hi_db: f32,
        lo_freq: f32,
        mid_freq: f32,
        hi_freq: f32,
        mid_q: f32,
        sr: f32,
    ) -> f32 {
        self.ensure_sr(sr);
        self.write_params(lo_db, mid_db, hi_db, lo_freq, mid_freq, hi_freq, mid_q);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        lo_db: f32,
        mid_db: f32,
        hi_db: f32,
        lo_freq: f32,
        mid_freq: f32,
        hi_freq: f32,
        mid_q: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(lo_db, mid_db, hi_db, lo_freq, mid_freq, hi_freq, mid_q);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustEq {
    fn default() -> Self {
        assert_slider_idx!(eq_dsp::EqDsp,
            "a_lo_db" => Self::LO_DB.0, "b_lo_f" => Self::LO_F.0,
            "c_mid_db" => Self::MID_DB.0, "d_mid_f" => Self::MID_F.0,
            "e_hi_db" => Self::HI_DB.0, "f_hi_f" => Self::HI_F.0,
            "g_mid_q" => Self::MID_Q.0);
        Self {
            dsp: eq_dsp::EqDsp::new(),
            sr: 0.0,
        }
    }
}

/// Allpass phaser (Julius Smith `pf.phaser2`), Faust-generated. One mono
/// instance per channel; [`FaustPhaser::new`] seeds the LFO phase per channel so
/// the notches sweep out of phase for stereo width. Mirrors the hand-written
/// `Phaser` interface. Sample-rate dependent (lazy re-init).
pub struct FaustPhaser {
    dsp: phaser_dsp::PhaserDsp,
    /// LFO phase offset in [0, 1], constant per channel.
    phase01: f32,
    /// Sample rate the Faust instance was initialized at (0.0 = not yet).
    sr: f32,
}

impl FaustPhaser {
    // Slider order in phaser.dsp (the a_/b_/.. prefixes force alphabetical to
    // equal this order): speed, fb, sweep, center, phase.
    const SPEED: ParamIndex = ParamIndex(0);
    const FB: ParamIndex = ParamIndex(1);
    const SWEEP: ParamIndex = ParamIndex(2);
    const CENTER: ParamIndex = ParamIndex(3);
    const PHASE: ParamIndex = ParamIndex(4);

    /// One phaser for channel `ch`. The right channel's LFO is offset a quarter
    /// cycle so the notch sweep is out of phase between channels (stereo width).
    pub fn new(ch: usize) -> Self {
        assert_slider_idx!(phaser_dsp::PhaserDsp,
            "a_speed" => Self::SPEED.0, "b_fb" => Self::FB.0, "c_sweep" => Self::SWEEP.0,
            "d_center" => Self::CENTER.0, "e_phase" => Self::PHASE.0);
        Self {
            dsp: phaser_dsp::PhaserDsp::new(),
            phase01: if ch == 1 { 0.25 } else { 0.0 },
            sr: 0.0,
        }
    }

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, rate: f32, depth: f32, center: f32, sweep: f32) {
        self.dsp.set_param(Self::SPEED, rate);
        self.dsp.set_param(Self::FB, depth);
        self.dsp.set_param(Self::SWEEP, sweep);
        self.dsp.set_param(Self::CENTER, center);
        self.dsp.set_param(Self::PHASE, self.phase01);
    }

    #[inline]
    pub fn process(&mut self, x: f32, rate: f32, depth: f32, center: f32, sweep: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(rate, depth, center, sweep);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(rate, depth, center, sweep);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

/// 3-voice stereo chorus, Faust-generated. Stereo in/out; mirrors the
/// hand-written `Chorus` interface (`process(l, r, rate, depth, delay, sr) ->
/// [f32; 2]`). Sample-rate dependent (lazy re-init).
pub struct FaustChorus {
    dsp: chorus_dsp::ChorusDsp,
    sr: f32,
}

impl FaustChorus {
    const RATE: ParamIndex = ParamIndex(0);
    const DEPTH: ParamIndex = ParamIndex(1);
    const DELAY: ParamIndex = ParamIndex(2);
    const TYPE: ParamIndex = ParamIndex(3);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, rate: f32, depth: f32, delay: f32, ctype: f32) {
        self.dsp.set_param(Self::RATE, rate);
        self.dsp.set_param(Self::DEPTH, depth);
        self.dsp.set_param(Self::DELAY, delay);
        self.dsp.set_param(Self::TYPE, ctype);
    }

    #[inline]
    pub fn process(
        &mut self,
        l: f32,
        r: f32,
        rate: f32,
        depth: f32,
        delay: f32,
        ctype: f32,
        sr: f32,
    ) -> [f32; 2] {
        self.ensure_sr(sr);
        self.write_params(rate, depth, delay, ctype);
        run_one_stereo(&mut self.dsp, l, r)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        rate: f32,
        depth: f32,
        delay: f32,
        ctype: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(rate, depth, delay, ctype);
        run_block_stereo(&mut self.dsp, buf, n);
    }
}

impl Default for FaustChorus {
    fn default() -> Self {
        assert_slider_idx!(chorus_dsp::ChorusDsp,
            "a_rate" => Self::RATE.0, "b_depth" => Self::DEPTH.0, "c_delay" => Self::DELAY.0,
            "d_type" => Self::TYPE.0);
        Self {
            dsp: chorus_dsp::ChorusDsp::new(),
            sr: 0.0,
        }
    }
}

/// LFO-swept flanger (feedback comb), Faust-generated. One mono instance per
/// channel; [`FaustFlanger::new`] seeds the LFO phase per channel so the sweep is
/// out of phase between channels (stereo width). Sample-rate dependent.
pub struct FaustFlanger {
    dsp: flanger_dsp::FlangerDsp,
    /// LFO phase offset in [0, 1], constant per channel.
    phase01: f32,
    sr: f32,
}

impl FaustFlanger {
    const RATE: ParamIndex = ParamIndex(0);
    const DEPTH: ParamIndex = ParamIndex(1);
    const FB: ParamIndex = ParamIndex(2);
    const PHASE: ParamIndex = ParamIndex(3);

    /// One flanger for channel `ch`; the right channel's LFO is offset a quarter
    /// cycle so the sweep is out of phase between channels (stereo width).
    pub fn new(ch: usize) -> Self {
        assert_slider_idx!(flanger_dsp::FlangerDsp,
            "a_rate" => Self::RATE.0, "b_depth" => Self::DEPTH.0,
            "c_fb" => Self::FB.0, "d_phase" => Self::PHASE.0);
        Self {
            dsp: flanger_dsp::FlangerDsp::new(),
            phase01: if ch == 1 { 0.25 } else { 0.0 },
            sr: 0.0,
        }
    }

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, rate: f32, depth: f32, fb: f32) {
        self.dsp.set_param(Self::RATE, rate);
        self.dsp.set_param(Self::DEPTH, depth);
        self.dsp.set_param(Self::FB, fb);
        self.dsp.set_param(Self::PHASE, self.phase01);
    }

    #[inline]
    pub fn process(&mut self, x: f32, rate: f32, depth: f32, fb: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(rate, depth, fb);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        rate: f32,
        depth: f32,
        fb: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(rate, depth, fb);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

/// Allpass-cascade smear / phase diffuser, Faust-generated. Mono per channel.
/// Mirrors the hand-written `Smear` interface. Sample-rate dependent.
pub struct FaustSmear {
    dsp: smear_dsp::SmearDsp,
    sr: f32,
}

impl FaustSmear {
    const MIX: ParamIndex = ParamIndex(0);
    const FREQ: ParamIndex = ParamIndex(1);
    const FB: ParamIndex = ParamIndex(2);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, mix: f32, freq: f32, fb: f32) {
        self.dsp.set_param(Self::MIX, mix);
        self.dsp.set_param(Self::FREQ, freq);
        self.dsp.set_param(Self::FB, fb);
    }

    #[inline]
    pub fn process(&mut self, x: f32, mix: f32, freq: f32, fb: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(mix, freq, fb);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        mix: f32,
        freq: f32,
        fb: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(mix, freq, fb);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustSmear {
    fn default() -> Self {
        assert_slider_idx!(smear_dsp::SmearDsp,
            "a_mix" => Self::MIX.0, "b_freq" => Self::FREQ.0, "c_fb" => Self::FB.0);
        Self {
            dsp: smear_dsp::SmearDsp::new(),
            sr: 0.0,
        }
    }
}

/// Haas placement delay, Faust-generated. A mono fractional delay the wrapper
/// applies to the right channel so the stereo image shifts. Mirrors the
/// hand-written `Haas` interface. Sample-rate dependent.
pub struct FaustHaas {
    dsp: haas_dsp::HaasDsp,
    sr: f32,
}

impl FaustHaas {
    const MS: ParamIndex = ParamIndex(0);
    /// The Haas delay is applied to the right channel.
    const CH: usize = 1;

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    /// Delay a single (right-channel) sample by the current `ms`.
    #[inline]
    pub fn process(&mut self, x: f32, ms: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::MS, ms);
        run_one(&mut self.dsp, x)
    }

    /// Delay the right channel of each frame in place.
    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ms: f32, sr: f32) {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::MS, ms);
        run_block(&mut self.dsp, buf, n, Self::CH);
    }
}

impl Default for FaustHaas {
    fn default() -> Self {
        assert_slider_idx!(haas_dsp::HaasDsp, "a_ms" => Self::MS.0);
        Self {
            dsp: haas_dsp::HaasDsp::new(),
            sr: 0.0,
        }
    }
}

/// 24 dB/oct state-variable filter (lp/hp/bp), Faust-generated — two cascaded
/// `fi.svf` stages. Mirrors the hand-written `SvfCascade` interface (public
/// `cutoff` field, `process` / `process_block(.., mode, q, sr)`) so voice
/// dispatch call sites are unchanged. Sample-rate dependent (lazy re-init).
pub struct FaustSvfCascade {
    /// Cutoff in Hz, set by the caller before each `process` / `process_block`.
    pub cutoff: f32,
    dsp: svf24_dsp::Svf24Dsp,
    sr: f32,
}

impl FaustSvfCascade {
    // Slider order in svf24.dsp (a_/b_/c_ prefixes): cutoff, q, mode.
    const CUTOFF: ParamIndex = ParamIndex(0);
    const Q: ParamIndex = ParamIndex(1);
    const MODE: ParamIndex = ParamIndex(2);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, mode: SvfMode, q: f32) {
        let mode_idx = match mode {
            SvfMode::Lp => 0.0,
            SvfMode::Hp => 1.0,
            SvfMode::Bp => 2.0,
        };
        self.dsp.set_param(Self::CUTOFF, self.cutoff);
        self.dsp.set_param(Self::Q, q);
        self.dsp.set_param(Self::MODE, mode_idx);
    }

    #[inline]
    pub fn process(&mut self, x: f32, mode: SvfMode, q: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(mode, q);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        mode: SvfMode,
        q: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(mode, q);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustSvfCascade {
    fn default() -> Self {
        assert_slider_idx!(svf24_dsp::Svf24Dsp,
            "a_cutoff" => Self::CUTOFF.0, "b_q" => Self::Q.0, "c_mode" => Self::MODE.0);
        Self {
            cutoff: 0.0,
            dsp: svf24_dsp::Svf24Dsp::new(),
            sr: 0.0,
        }
    }
}

/// Moog-style multimode ladder filter (lp/hp/bp), Faust-generated. Mirrors the
/// hand-written `LadderFilter` interface (`process(x, cutoff, resonance, mode,
/// sr)`) so voice dispatch call sites are unchanged. Sample-rate dependent.
pub struct FaustLadder {
    dsp: ladder_dsp::LadderDsp,
    sr: f32,
}

impl FaustLadder {
    // Slider order in ladder.dsp (a_/b_/c_ prefixes): cutoff, q, mode.
    const CUTOFF: ParamIndex = ParamIndex(0);
    const Q: ParamIndex = ParamIndex(1);
    const MODE: ParamIndex = ParamIndex(2);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, cutoff: f32, resonance: f32, mode: LadderMode) {
        let mode_idx = match mode {
            LadderMode::Lp => 0.0,
            LadderMode::Hp => 1.0,
            LadderMode::Bp => 2.0,
        };
        self.dsp.set_param(Self::CUTOFF, cutoff);
        self.dsp.set_param(Self::Q, resonance);
        self.dsp.set_param(Self::MODE, mode_idx);
    }

    #[inline]
    pub fn process(
        &mut self,
        x: f32,
        cutoff: f32,
        resonance: f32,
        mode: LadderMode,
        sr: f32,
    ) -> f32 {
        self.ensure_sr(sr);
        self.write_params(cutoff, resonance, mode);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        cutoff: f32,
        resonance: f32,
        mode: LadderMode,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(cutoff, resonance, mode);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustLadder {
    fn default() -> Self {
        assert_slider_idx!(ladder_dsp::LadderDsp,
            "a_cutoff" => Self::CUTOFF.0, "b_q" => Self::Q.0, "c_mode" => Self::MODE.0);
        Self {
            dsp: ladder_dsp::LadderDsp::new(),
            sr: 0.0,
        }
    }
}

/// Vital's reverb (`re.vital_rev`), Faust-generated — an orbit-bus send effect.
/// Stereo 2-in/2-out, block-rate. Initialized at the orbit's sample rate in
/// `new(sr)`; runs fully wet (the `.dsp` hardcodes `mix=1.0`) since the orbit
/// scales the input by the send level and adds the wet back onto the bus.
pub struct FaustVitalRev {
    dsp: Box<vital_rev_dsp::VitalRevDsp>,
}

impl FaustVitalRev {
    // Slider order in vital_rev.dsp (a_/b_/.. prefixes).
    const PRELOW: ParamIndex = ParamIndex(0);
    const PREHIGH: ParamIndex = ParamIndex(1);
    const LOWCUT: ParamIndex = ParamIndex(2);
    const HIGHCUT: ParamIndex = ParamIndex(3);
    const LOWGAIN: ParamIndex = ParamIndex(4);
    const HIGHGAIN: ParamIndex = ParamIndex(5);
    const CHORUS: ParamIndex = ParamIndex(6);
    const CHORUSFREQ: ParamIndex = ParamIndex(7);
    const PREDELAY: ParamIndex = ParamIndex(8);
    const TIME: ParamIndex = ParamIndex(9);
    const SIZE: ParamIndex = ParamIndex(10);

    pub fn new(sr: f32) -> Self {
        assert_slider_idx!(vital_rev_dsp::VitalRevDsp,
            "a_prelow" => Self::PRELOW.0, "b_prehigh" => Self::PREHIGH.0,
            "c_lowcut" => Self::LOWCUT.0, "d_highcut" => Self::HIGHCUT.0,
            "e_lowgain" => Self::LOWGAIN.0, "f_highgain" => Self::HIGHGAIN.0,
            "g_chorus" => Self::CHORUS.0, "h_chorusfreq" => Self::CHORUSFREQ.0,
            "i_predelay" => Self::PREDELAY.0, "j_time" => Self::TIME.0, "k_size" => Self::SIZE.0);
        let mut dsp = boxed_zeroed::<vital_rev_dsp::VitalRevDsp>();
        dsp.init(sr as i32);
        Self { dsp }
    }

    #[inline]
    pub fn process_block(&mut self, frames: &mut [StereoFrame], n: usize, p: &ReverbParams) {
        self.dsp.set_param(Self::PRELOW, p.prelow);
        self.dsp.set_param(Self::PREHIGH, p.prehigh);
        self.dsp.set_param(Self::LOWCUT, p.lowcut);
        self.dsp.set_param(Self::HIGHCUT, p.highcut);
        self.dsp.set_param(Self::LOWGAIN, p.lowgain);
        self.dsp.set_param(Self::HIGHGAIN, p.highgain);
        self.dsp.set_param(Self::CHORUS, p.chorus);
        self.dsp.set_param(Self::CHORUSFREQ, p.chorus_freq);
        self.dsp.set_param(Self::PREDELAY, p.predelay);
        self.dsp.set_param(Self::TIME, p.decay);
        self.dsp.set_param(Self::SIZE, p.size);
        run_block_stereo(&mut *self.dsp, frames, n);
    }
}

/// Julian Parker's lush ambient reverb (`re.jpverb`), Faust-generated — an
/// orbit-bus send effect. Stereo 2-in/2-out, block-rate, initialized in
/// `new(sr)`. The `.dsp` remaps the orbit's 0..1 params to jpverb's ranges.
pub struct FaustJpVerb {
    dsp: Box<jpverb_dsp::JpverbDsp>,
}

impl FaustJpVerb {
    // Slider order in jpverb.dsp (a_/b_/.. prefixes).
    const DECAY: ParamIndex = ParamIndex(0);
    const DAMP: ParamIndex = ParamIndex(1);
    const SIZE: ParamIndex = ParamIndex(2);
    const DIFF: ParamIndex = ParamIndex(3);
    const MODDEPTH: ParamIndex = ParamIndex(4);
    const MODFREQ: ParamIndex = ParamIndex(5);
    const LOW: ParamIndex = ParamIndex(6);
    const HIGH: ParamIndex = ParamIndex(7);
    const LOWCUT: ParamIndex = ParamIndex(8);
    const HIGHCUT: ParamIndex = ParamIndex(9);

    pub fn new(sr: f32) -> Self {
        assert_slider_idx!(jpverb_dsp::JpverbDsp,
            "a_decay" => Self::DECAY.0, "b_damp" => Self::DAMP.0, "c_size" => Self::SIZE.0,
            "d_diff" => Self::DIFF.0, "e_moddepth" => Self::MODDEPTH.0, "f_modfreq" => Self::MODFREQ.0,
            "g_low" => Self::LOW.0, "h_high" => Self::HIGH.0,
            "i_lowcut" => Self::LOWCUT.0, "j_highcut" => Self::HIGHCUT.0);
        let mut dsp = boxed_zeroed::<jpverb_dsp::JpverbDsp>();
        dsp.init(sr as i32);
        Self { dsp }
    }

    #[inline]
    pub fn process_block(&mut self, frames: &mut [StereoFrame], n: usize, p: &ReverbParams) {
        self.dsp.set_param(Self::DECAY, p.decay);
        self.dsp.set_param(Self::DAMP, p.damp);
        self.dsp.set_param(Self::SIZE, p.size);
        self.dsp.set_param(Self::DIFF, p.diff);
        self.dsp.set_param(Self::MODDEPTH, p.chorus);
        self.dsp.set_param(Self::MODFREQ, p.chorus_freq);
        self.dsp.set_param(Self::LOW, p.lowgain);
        self.dsp.set_param(Self::HIGH, p.highgain);
        self.dsp.set_param(Self::LOWCUT, p.lowcut);
        self.dsp.set_param(Self::HIGHCUT, p.highcut);
        run_block_stereo(&mut *self.dsp, frames, n);
    }
}
