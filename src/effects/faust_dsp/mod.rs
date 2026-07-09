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

use super::comb::CombParams;
use super::delay::DelayParams;
use super::feedback::FeedbackParams;
use super::reverb::ReverbParams;
use super::LadderMode;
use crate::dsp::SvfMode;
use crate::types::{DelayType, StereoFrame, MAX_BLOCK};
use faust_types::{FaustDsp, ParamIndex, UI};
use std::mem::MaybeUninit;

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
mod fshift_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("fshift_gen.rs");
}
mod pshift_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("pshift_gen.rs");
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
mod comb_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("comb_gen.rs");
}
mod feedback_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("feedback_gen.rs");
}
// Four standalone delay algorithms (split from the old single delay.dsp so the
// FaustDelay wrapper runs only the selected one — ~4x less CPU). Each is a 2-in/
// 2-out DSP with the same a_time/b_fb slider order.
mod delay_standard_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("delay_standard_gen.rs");
}
mod delay_pingpong_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("delay_pingpong_gen.rs");
}
mod delay_tape_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("delay_tape_gen.rs");
}
mod delay_multitap_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("delay_multitap_gen.rs");
}
mod wah_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("wah_gen.rs");
}
mod vinyl_dsp {
    #![allow(clippy::all, warnings)]
    use faust_types::*;
    include!("vinyl_gen.rs");
}

/// Block scratch for the `run_block*` helpers: `MAX_BLOCK`-sized stack arrays
/// initialized only over `[..n]`. A plain `[0.0; MAX_BLOCK]` local would memset
/// the full 1 KB per array per call — an 8× over-zero at the default n = 32.
type BlockScratch = [MaybeUninit<f32>; MAX_BLOCK];

#[inline]
fn block_scratch() -> BlockScratch {
    [MaybeUninit::uninit(); MAX_BLOCK]
}

/// # Safety
/// Every element of `s` must have been initialized.
#[inline]
unsafe fn init_slice(s: &[MaybeUninit<f32>]) -> &[f32] {
    std::slice::from_raw_parts(s.as_ptr().cast::<f32>(), s.len())
}

/// # Safety
/// Every element of `s` must have been initialized.
#[inline]
unsafe fn init_slice_mut(s: &mut [MaybeUninit<f32>]) -> &mut [f32] {
    std::slice::from_raw_parts_mut(s.as_mut_ptr().cast::<f32>(), s.len())
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
    let mut input = block_scratch();
    let mut output = block_scratch();
    for i in 0..n {
        input[i].write(buf[i][ch]);
        output[i].write(0.0);
    }
    // SAFETY: `[..n]` of both arrays was initialized by the loop above.
    let (input, output) = unsafe { (init_slice(&input[..n]), init_slice_mut(&mut output[..n])) };
    dsp.compute(n as i32, &[input], &mut [&mut *output]);
    for i in 0..n {
        buf[i][ch] = output[i];
    }
}

/// Run a mono Faust DSP whose FIRST input is a per-sample control signal and
/// second input is the audio in `buf` (a flat mono slice, the per-channel comb's
/// send scratch on the orbit bus), over `n` samples in place. `ctrl` must be at
/// least `n` long. The flat-mono counterpart of [`run_block_stereo_mod`] for
/// DSPs that take a click-sensitive param as input 0.
#[inline]
fn run_block_flat_mod<D: FaustDsp<T = f32>>(dsp: &mut D, buf: &mut [f32], n: usize, ctrl: &[f32]) {
    let mut input = block_scratch();
    let mut output = block_scratch();
    for i in 0..n {
        input[i].write(buf[i]);
        output[i].write(0.0);
    }
    // SAFETY: `[..n]` of both arrays was initialized by the loop above.
    let (input, output) = unsafe { (init_slice(&input[..n]), init_slice_mut(&mut output[..n])) };
    dsp.compute(n as i32, &[&ctrl[..n], input], &mut [&mut *output]);
    buf[..n].copy_from_slice(output);
}

/// Run a stereo (2-in/2-out audio) Faust DSP whose FIRST input is a per-sample
/// control signal, so the actual input order is `[ctrl, l, r]`, over `n` frames
/// of `buf` in place. `ctrl` must be at least `n` long. The audio-rate
/// counterpart of [`run_block_stereo`] for DSPs that take a click-sensitive
/// param as input 0.
#[inline]
fn run_block_stereo_mod<D: FaustDsp<T = f32>>(
    dsp: &mut D,
    buf: &mut [StereoFrame],
    n: usize,
    ctrl: &[f32],
) {
    let mut in_l = block_scratch();
    let mut in_r = block_scratch();
    let mut out_l = block_scratch();
    let mut out_r = block_scratch();
    for i in 0..n {
        in_l[i].write(buf[i][0]);
        in_r[i].write(buf[i][1]);
        out_l[i].write(0.0);
        out_r[i].write(0.0);
    }
    // SAFETY: `[..n]` of all four arrays was initialized by the loop above.
    let (in_l, in_r, out_l, out_r) = unsafe {
        (
            init_slice(&in_l[..n]),
            init_slice(&in_r[..n]),
            init_slice_mut(&mut out_l[..n]),
            init_slice_mut(&mut out_r[..n]),
        )
    };
    dsp.compute(
        n as i32,
        &[&ctrl[..n], in_l, in_r],
        &mut [&mut *out_l, &mut *out_r],
    );
    for i in 0..n {
        buf[i][0] = out_l[i];
        buf[i][1] = out_r[i];
    }
}

/// Run a stereo (2-in/2-out) Faust DSP for one frame. Params set beforehand.
#[inline]
fn run_one_stereo<D: FaustDsp<T = f32>>(dsp: &mut D, l: f32, r: f32) -> [f32; 2] {
    let in_l = [l];
    let in_r = [r];
    let mut out_l = [0.0f32];
    let mut out_r = [0.0f32];
    dsp.compute(
        1,
        &[&in_l[..], &in_r[..]],
        &mut [&mut out_l[..], &mut out_r[..]],
    );
    [out_l[0], out_r[0]]
}

/// Run a stereo (2-in/2-out) Faust DSP over `n` frames of `buf`, in place.
/// Params (held for the block) set beforehand.
#[inline]
fn run_block_stereo<D: FaustDsp<T = f32>>(dsp: &mut D, buf: &mut [StereoFrame], n: usize) {
    let mut in_l = block_scratch();
    let mut in_r = block_scratch();
    let mut out_l = block_scratch();
    let mut out_r = block_scratch();
    for i in 0..n {
        in_l[i].write(buf[i][0]);
        in_r[i].write(buf[i][1]);
        out_l[i].write(0.0);
        out_r[i].write(0.0);
    }
    // SAFETY: `[..n]` of all four arrays was initialized by the loop above.
    let (in_l, in_r, out_l, out_r) = unsafe {
        (
            init_slice(&in_l[..n]),
            init_slice(&in_r[..n]),
            init_slice_mut(&mut out_l[..n]),
            init_slice_mut(&mut out_r[..n]),
        )
    };
    dsp.compute(n as i32, &[in_l, in_r], &mut [&mut *out_l, &mut *out_r]);
    for i in 0..n {
        buf[i][0] = out_l[i];
        buf[i][1] = out_r[i];
    }
}

/// Allocate a Faust DSP directly on the heap, zeroed, without ever materializing
/// it on the stack. The reverb DSPs carry multi-megabyte inline delay arrays, so
/// by-value construction (`Box::new(D::new())`) overflows the stack in debug.
/// Faust's own `new()` only zeroes every field, so a zeroed heap struct is
/// equivalent — pair with [`init_zeroed`] to finish initialization.
///
/// Uses `alloc_zeroed` (not `new_uninit` + memset): the allocator satisfies a
/// large zeroed request with demand-zero pages, so the delay arrays consume no
/// physical memory until the effect actually writes them.
#[inline]
fn boxed_zeroed<D>() -> Box<D> {
    let layout = std::alloc::Layout::new::<D>();
    assert!(layout.size() != 0, "Faust DSP structs are never zero-sized");
    // SAFETY: `alloc_zeroed` uses the same global allocator `Box` frees with,
    // at `D`'s layout. Every generated Faust DSP struct holds only `i32`/`f32`
    // scalars and arrays, for which the all-zero bit pattern is the valid
    // initial value Faust's `new()` writes.
    unsafe {
        let ptr = std::alloc::alloc_zeroed(layout).cast::<D>();
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Box::from_raw(ptr)
    }
}

/// Finish initializing a DSP freshly allocated by [`boxed_zeroed`], skipping
/// the `instance_clear` that a full `init()` would run: on an all-zero struct
/// the clear only rewrites zeros, but doing so touches every page of the delay
/// arrays and defeats the demand-zero allocation.
#[inline]
fn init_zeroed<D: FaustDsp<T = f32>>(dsp: &mut D, sr: i32) {
    D::class_init(sr);
    dsp.instance_constants(sr);
    dsp.instance_reset_params();
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

/// Envelope-follower auto-wah (resonant bandpass swept by the input envelope),
/// Faust-generated. One mono instance per channel. Sample-rate dependent (the
/// follower + SVF), so it lazily (re)inits when the rate arrives or changes.
pub struct FaustWah {
    dsp: wah_dsp::WahDsp,
    sr: f32,
}

impl FaustWah {
    const WAH: ParamIndex = ParamIndex(0);
    const PEAK: ParamIndex = ParamIndex(1);
    const SENS: ParamIndex = ParamIndex(2);
    const MANUAL: ParamIndex = ParamIndex(3);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, amount: f32, peak: f32, sens: f32, manual: f32) {
        self.dsp.set_param(Self::WAH, amount);
        self.dsp.set_param(Self::PEAK, peak);
        self.dsp.set_param(Self::SENS, sens);
        self.dsp.set_param(Self::MANUAL, manual);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        x: f32,
        amount: f32,
        peak: f32,
        sens: f32,
        manual: f32,
        sr: f32,
    ) -> f32 {
        self.ensure_sr(sr);
        self.write_params(amount, peak, sens, manual);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        peak: f32,
        sens: f32,
        manual: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(amount, peak, sens, manual);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustWah {
    fn default() -> Self {
        assert_slider_idx!(wah_dsp::WahDsp,
            "a_wah" => Self::WAH.0, "b_peak" => Self::PEAK.0,
            "c_sens" => Self::SENS.0, "d_manual" => Self::MANUAL.0);
        Self {
            dsp: wah_dsp::WahDsp::new(),
            sr: 0.0,
        }
    }
}

/// Single-sideband frequency shifter (analytic-signal heterodyne: `fi.pospass`
/// Hilbert pair × `os.quadosc` carrier), Faust-generated. One mono instance per
/// channel. The signed shift in Hz selects the sideband (>0 up, <0 down).
/// Sample-rate dependent (the pospass filters + the carrier), so it lazily
/// (re)inits when the rate arrives or changes.
pub struct FaustFreqShift {
    dsp: fshift_dsp::FreqShiftDsp,
    sr: f32,
}

impl FaustFreqShift {
    const SHIFT: ParamIndex = ParamIndex(0);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32, shift: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::SHIFT, shift);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        shift: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.dsp.set_param(Self::SHIFT, shift);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustFreqShift {
    fn default() -> Self {
        assert_slider_idx!(fshift_dsp::FreqShiftDsp, "a_shift" => Self::SHIFT.0);
        Self {
            dsp: fshift_dsp::FreqShiftDsp::new(),
            sr: 0.0,
        }
    }
}

/// Granular (delay-line) pitch shifter via `ef.transpose`, Faust-generated. One
/// mono instance per channel. `shift` transposes in semitones (signed); `window`
/// is the grain length in ms — the warble/character knob. Sample-rate dependent
/// (the window is ms → samples via `ma.SR`), so it lazily (re)inits when the rate
/// arrives or changes.
pub struct FaustPitchShift {
    /// Boxed: the 512 KB delay line would otherwise sit inline in every
    /// voice's `VoiceFxState`. Allocated demand-zero (`boxed_zeroed`), so a
    /// voice that never pitch-shifts never makes it resident.
    dsp: Box<pshift_dsp::PitchShiftDsp>,
    /// Sample rate the instance was initialized at. Invariant: `0.0` means
    /// the delay line is all-zero (fresh allocation or just cleared by
    /// `reset_in_place`), which lets `ensure_sr` skip `instance_clear` — the
    /// clear would rewrite 512 KB of zeros and fault in every untouched page.
    sr: f32,
}

impl FaustPitchShift {
    const SHIFT: ParamIndex = ParamIndex(0);
    const WINDOW: ParamIndex = ParamIndex(1);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            if self.sr == 0.0 {
                // State is all-zero (see the `sr` invariant): full `init()`
                // minus the redundant page-touching clear.
                init_zeroed(&mut *self.dsp, sr as i32);
            } else {
                self.dsp.init(sr as i32);
            }
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, shift: f32, window: f32) {
        self.dsp.set_param(Self::SHIFT, shift);
        self.dsp.set_param(Self::WINDOW, window);
    }

    #[inline]
    pub fn process(&mut self, x: f32, shift: f32, window: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(shift, window);
        run_one(&mut *self.dsp, x)
    }

    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        shift: f32,
        window: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(shift, window);
        run_block(&mut *self.dsp, buf, n, ch);
    }

    /// Clear the 512 KB delay line in place. Rebuilding via `Default` would
    /// materialize that buffer as a stack temporary — fine on the main thread,
    /// fatal on the audio thread's small stack (note-on `reset` runs there).
    /// `sr = 0.0` forces a re-init on the next `process`. Skipped entirely
    /// when `sr` is already `0.0`: by the field invariant the line is still
    /// all-zero, and the memset would cost half a megabyte per note-on.
    pub fn reset_in_place(&mut self) {
        if self.sr != 0.0 {
            self.dsp.instance_clear();
            self.sr = 0.0;
        }
    }
}

impl Default for FaustPitchShift {
    fn default() -> Self {
        assert_slider_idx!(pshift_dsp::PitchShiftDsp,
            "a_shift" => Self::SHIFT.0, "b_window" => Self::WINDOW.0);
        Self {
            dsp: boxed_zeroed::<pshift_dsp::PitchShiftDsp>(),
            sr: 0.0,
        }
    }
}

/// VinylSim / Cassette "character" insert (wow+flutter, band-limit, hiss, sat),
/// Faust-generated. One mono instance per channel. Sample-rate dependent (LFOs +
/// filters), so it lazily (re)inits when the rate arrives or changes.
pub struct FaustVinyl {
    dsp: vinyl_dsp::VinylDsp,
    sr: f32,
}

impl FaustVinyl {
    const VINYL: ParamIndex = ParamIndex(0);
    const WOW: ParamIndex = ParamIndex(1);
    const NOISE: ParamIndex = ParamIndex(2);
    const TONE: ParamIndex = ParamIndex(3);
    const TYPE: ParamIndex = ParamIndex(4);

    #[inline]
    fn ensure_sr(&mut self, sr: f32) {
        if self.sr != sr {
            self.dsp.init(sr as i32);
            self.sr = sr;
        }
    }

    #[inline]
    fn write_params(&mut self, amount: f32, wow: f32, noise: f32, tone: f32, kind: f32) {
        self.dsp.set_param(Self::VINYL, amount);
        self.dsp.set_param(Self::WOW, wow);
        self.dsp.set_param(Self::NOISE, noise);
        self.dsp.set_param(Self::TONE, tone);
        self.dsp.set_param(Self::TYPE, kind);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        x: f32,
        amount: f32,
        wow: f32,
        noise: f32,
        tone: f32,
        kind: f32,
        sr: f32,
    ) -> f32 {
        self.ensure_sr(sr);
        self.write_params(amount, wow, noise, tone, kind);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        wow: f32,
        noise: f32,
        tone: f32,
        kind: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(amount, wow, noise, tone, kind);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustVinyl {
    fn default() -> Self {
        assert_slider_idx!(vinyl_dsp::VinylDsp,
            "a_vinyl" => Self::VINYL.0, "b_wow" => Self::WOW.0, "c_noise" => Self::NOISE.0,
            "d_tone" => Self::TONE.0, "e_type" => Self::TYPE.0);
        Self {
            dsp: vinyl_dsp::VinylDsp::new(),
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
    const ASYM: ParamIndex = ParamIndex(3);

    #[inline]
    fn write_params(&mut self, amount: f32, postgain: f32, mode: f32, asym: f32) {
        self.dsp.set_param(Self::DISTORT, amount);
        self.dsp.set_param(Self::DISTORTVOL, postgain);
        self.dsp.set_param(Self::DISTORTMODE, mode);
        self.dsp.set_param(Self::ASYM, asym);
    }

    #[inline]
    pub fn process(&mut self, x: f32, amount: f32, postgain: f32, mode: f32, asym: f32) -> f32 {
        self.write_params(amount, postgain, mode, asym);
        run_one(&mut self.dsp, x)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        ch: usize,
        amount: f32,
        postgain: f32,
        mode: f32,
        asym: f32,
    ) {
        self.write_params(amount, postgain, mode, asym);
        run_block(&mut self.dsp, buf, n, ch);
    }
}

impl Default for FaustDistort {
    fn default() -> Self {
        assert_slider_idx!(distort_dsp::DistortDsp,
            "a_distort" => Self::DISTORT.0, "b_distortvol" => Self::DISTORTVOL.0,
            "c_distortmode" => Self::DISTORTMODE.0, "d_asym" => Self::ASYM.0);
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
    pub fn process(
        &mut self,
        x: f32,
        rate: f32,
        depth: f32,
        center: f32,
        sweep: f32,
        sr: f32,
    ) -> f32 {
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
    const THRU: ParamIndex = ParamIndex(4);

    /// One flanger for channel `ch`; the right channel's LFO is offset a quarter
    /// cycle so the sweep is out of phase between channels (stereo width).
    pub fn new(ch: usize) -> Self {
        assert_slider_idx!(flanger_dsp::FlangerDsp,
            "a_rate" => Self::RATE.0, "b_depth" => Self::DEPTH.0,
            "c_fb" => Self::FB.0, "d_phase" => Self::PHASE.0, "e_thru" => Self::THRU.0);
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
    fn write_params(&mut self, rate: f32, depth: f32, fb: f32, mode: f32) {
        self.dsp.set_param(Self::RATE, rate);
        self.dsp.set_param(Self::DEPTH, depth);
        self.dsp.set_param(Self::FB, fb);
        self.dsp.set_param(Self::PHASE, self.phase01);
        self.dsp.set_param(Self::THRU, mode);
    }

    #[inline]
    pub fn process(&mut self, x: f32, rate: f32, depth: f32, fb: f32, mode: f32, sr: f32) -> f32 {
        self.ensure_sr(sr);
        self.write_params(rate, depth, fb, mode);
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
        mode: f32,
        sr: f32,
    ) {
        self.ensure_sr(sr);
        self.write_params(rate, depth, fb, mode);
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
        init_zeroed(&mut *dsp, sr as i32);
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
        init_zeroed(&mut *dsp, sr as i32);
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

/// Feedback comb resonator (Karplus-Strong style), Faust-generated — a per-orbit
/// bus send effect. Mono (1-in/1-out), one instance per channel, mirroring the
/// hand-written `effects::comb::Comb`. Initialized at the orbit sample rate;
/// `combfreq` derives the delay length from `ma.SR` inside the DSP.
pub struct FaustComb {
    dsp: comb_dsp::CombDsp,
}

impl FaustComb {
    // Slider order in comb.dsp (b_/c_ prefixes). `freq` is an audio-rate input
    // (channel 0), not a slider, so the gain params start at index 0.
    const FB: ParamIndex = ParamIndex(0);
    const DAMP: ParamIndex = ParamIndex(1);

    pub fn new(sr: f32) -> Self {
        assert_slider_idx!(comb_dsp::CombDsp, "b_fb" => Self::FB.0, "c_damp" => Self::DAMP.0);
        let mut dsp = comb_dsp::CombDsp::new();
        dsp.init(sr as i32);
        Self { dsp }
    }

    /// Process `n` samples in place. `freq` is the per-sample fundamental (Hz),
    /// fed as Faust input 0 so a swept ModChain stays audio-rate (no pitch-zipper).
    #[inline]
    pub fn process_block(&mut self, buf: &mut [f32], n: usize, p: &CombParams, freq: &[f32]) {
        self.dsp.set_param(Self::FB, p.feedback);
        self.dsp.set_param(Self::DAMP, p.damp);
        run_block_flat_mod(&mut self.dsp, buf, n, freq);
    }
}

/// Stereo feedback delay (cross-channel + LFO-modulated time), Faust-generated —
/// a per-orbit bus send effect. Replaces `effects::feedback::Feedback`. Stereo
/// 2-in/2-out, block-rate, initialized in `new(sr)`. The orbit's send level is
/// passed in as `fb_amount` (the re-injection coefficient), matching the native
/// call. Boxed: the 1 s stereo delay lines are multi-megabyte.
pub struct FaustFeedback {
    dsp: Box<feedback_dsp::FeedbackDsp>,
}

impl FaustFeedback {
    // Slider order in feedback.dsp (b_/c_/g_ prefixes). `time` is an audio-rate
    // input (channel 0), not a slider, so the remaining params start at index 0.
    const DAMP: ParamIndex = ParamIndex(0);
    const CROSS: ParamIndex = ParamIndex(1);
    const FB: ParamIndex = ParamIndex(2);

    pub fn new(sr: f32) -> Self {
        assert_slider_idx!(feedback_dsp::FeedbackDsp,
            "b_damp" => Self::DAMP.0, "c_cross" => Self::CROSS.0, "g_fb" => Self::FB.0);
        let mut dsp = boxed_zeroed::<feedback_dsp::FeedbackDsp>();
        init_zeroed(&mut *dsp, sr as i32);
        Self { dsp }
    }

    /// Process `n` stereo frames in place. `time` is the per-sample base delay
    /// (ms); it is reciprocated to a delay frequency (1000/ms) and fed as Faust
    /// input 0, the only form Faust can statically size the delay line from
    /// (`ds = ma.SR/dfreq`, see feedback.dsp header). A swept ModChain stays
    /// audio-rate. `fb_amount` is the orbit send level doubling as the
    /// re-injection coefficient.
    #[inline]
    pub fn process_block(
        &mut self,
        buf: &mut [StereoFrame],
        n: usize,
        p: &FeedbackParams,
        fb_amount: f32,
        time: &[f32],
    ) {
        self.dsp.set_param(Self::DAMP, p.damp);
        self.dsp.set_param(Self::CROSS, p.cross);
        self.dsp.set_param(Self::FB, fb_amount);
        let mut dfreq = block_scratch();
        for (d, &t) in dfreq.iter_mut().zip(time.iter()).take(n) {
            d.write(1000.0 / t.max(0.01));
        }
        // SAFETY: `[..n]` was initialized by the loop above (`time` is ≥ n long).
        let dfreq = unsafe { init_slice(&dfreq[..n]) };
        run_block_stereo_mod(&mut *self.dsp, buf, n, dfreq);
    }
}

/// Stereo multi-algorithm delay (standard/pingpong/tape/multitap), Faust-
/// generated — a per-orbit bus send effect. Replaces `effects::delay::Delay`.
/// Stereo 2-in/2-out, block-rate, initialized in `new(sr)`. Holds one boxed DSP
/// per algorithm and runs ONLY the one `delaytype` selects (the old single DSP
/// ran all four every block — ~4x the CPU). All four stay resident, so switching
/// type never allocates; the inactive instances' tails freeze until reselected.
/// Boxed: the four 65 k-sample stereo lines are ~2 MB total (same as before).
pub struct FaustDelay {
    standard: Box<delay_standard_dsp::DelayStandardDsp>,
    pingpong: Box<delay_pingpong_dsp::DelayPingpongDsp>,
    tape: Box<delay_tape_dsp::DelayTapeDsp>,
    multitap: Box<delay_multitap_dsp::DelayMultitapDsp>,
}

impl FaustDelay {
    // Slider order is identical in all four split DSPs (a_time, b_fb).
    const TIME: ParamIndex = ParamIndex(0);
    const FB: ParamIndex = ParamIndex(1);

    pub fn new(sr: f32) -> Self {
        assert_slider_idx!(delay_standard_dsp::DelayStandardDsp,
            "a_time" => Self::TIME.0, "b_fb" => Self::FB.0);
        assert_slider_idx!(delay_pingpong_dsp::DelayPingpongDsp,
            "a_time" => Self::TIME.0, "b_fb" => Self::FB.0);
        assert_slider_idx!(delay_tape_dsp::DelayTapeDsp,
            "a_time" => Self::TIME.0, "b_fb" => Self::FB.0);
        assert_slider_idx!(delay_multitap_dsp::DelayMultitapDsp,
            "a_time" => Self::TIME.0, "b_fb" => Self::FB.0);
        let mut standard = boxed_zeroed::<delay_standard_dsp::DelayStandardDsp>();
        let mut pingpong = boxed_zeroed::<delay_pingpong_dsp::DelayPingpongDsp>();
        let mut tape = boxed_zeroed::<delay_tape_dsp::DelayTapeDsp>();
        let mut multitap = boxed_zeroed::<delay_multitap_dsp::DelayMultitapDsp>();
        init_zeroed(&mut *standard, sr as i32);
        init_zeroed(&mut *pingpong, sr as i32);
        init_zeroed(&mut *tape, sr as i32);
        init_zeroed(&mut *multitap, sr as i32);
        Self {
            standard,
            pingpong,
            tape,
            multitap,
        }
    }

    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, p: &DelayParams) {
        match p.delay_type {
            DelayType::Standard => {
                self.standard.set_param(Self::TIME, p.time);
                self.standard.set_param(Self::FB, p.feedback);
                run_block_stereo(&mut *self.standard, buf, n);
            }
            DelayType::PingPong => {
                self.pingpong.set_param(Self::TIME, p.time);
                self.pingpong.set_param(Self::FB, p.feedback);
                run_block_stereo(&mut *self.pingpong, buf, n);
            }
            DelayType::Tape => {
                self.tape.set_param(Self::TIME, p.time);
                self.tape.set_param(Self::FB, p.feedback);
                run_block_stereo(&mut *self.tape, buf, n);
            }
            DelayType::Multitap => {
                self.multitap.set_param(Self::TIME, p.time);
                self.multitap.set_param(Self::FB, p.feedback);
                run_block_stereo(&mut *self.multitap, buf, n);
            }
        }
    }
}
