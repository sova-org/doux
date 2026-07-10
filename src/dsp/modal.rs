//! Modal resonator bank for metallic percussion (hat, cymbal).
//!
//! A fixed bank of `MODAL_MODES` high-Q bandpass resonators tuned to inharmonic
//! ratios, each with its own decay rate. Noise (or impulse) excitation through
//! the bank reads as evolving metal: the modes bloom together on the strike then
//! die at different rates, so the timbre shifts over the tail instead of sitting
//! as a static filtered buzz.
//!
//! One bank per voice (a voice is one instrument at a time). Per-channel state so
//! the two channels can be excited by decorrelated noise for stereo width. All
//! state is pre-allocated; [`ModalBank::reset_in_place`] zeroes the filters
//! without reallocating (the bank is boxed once at voice construction).
//!
//! Each mode is the bandpass tap of a trapezoidal (tan-prewarped) SVF — the same
//! topology as the Faust `svf.dsp` used elsewhere in the voice — but with its
//! coefficients cached and recomputed only when that mode's cutoff (or the
//! shared Q / sample rate) actually changes. Tunings are per-trigger constants
//! in practice, so the steady-state cost is ~6 mul-adds per mode instead of a
//! `tan` plus three divides per sample.

use std::f32::consts::PI;

use crate::dsp::decay;
use crate::types::CHANNELS;

/// Number of resonators per channel. 16 is dense enough to read as real metal;
/// tunable (drop it if the 16×2 = 32 BP/sample cost is hot on the target).
pub const MODAL_MODES: usize = 16;

/// Per-mode decay-rate law: mode `m` decays at `base_decay · (1 + STEP·m)`, so
/// higher modes die faster (physically real — high partials of a struck plate
/// go first) and one scalar controls tail length. The law being linear in `m`
/// is what lets [`ModalBank::process`] build all the envelopes from two
/// `exp2f` calls and a running multiply: `e^{-t·r(1+s·m)} = e^{-t·r}·(e^{-t·r·s})^m`.
const MODE_DECAY_STEP: f32 = 0.4;

/// Tuning of the bank for one [`ModalBank::process`] call. Per-trigger
/// constants in practice, but sampled every call so per-sample modulation
/// (e.g. of the ratio spread) still tracks.
pub struct ModalParams<'a> {
    /// Base frequency in Hz; mode `m` sits at `base · ratios[m]`.
    pub base: f32,
    /// Inharmonic center-frequency ratios, one per mode.
    pub ratios: &'a [f32; MODAL_MODES],
    /// Base decay rate in nepers/s; see [`MODE_DECAY_STEP`].
    pub base_decay: f32,
    /// Shared resonator Q (the real filter Q, clamped to the Faust SVF's
    /// `[0.5, 30.5]` range when coefficients are computed).
    pub q: f32,
}

/// Sentinel for [`Mode::cutoff`] marking cached coefficients as invalid
/// (real cutoffs are ≥ 0, so this never matches a requested one).
const UNTUNED: f32 = -1.0;

/// One bandpass resonator: trapezoidal-SVF state plus coefficients cached for
/// the `(cutoff, q, sr)` they were computed at. `cutoff` is the cache key; the
/// bank tracks the shared `(q, sr)` pair itself.
#[derive(Clone, Copy)]
struct Mode {
    /// Requested (unclamped) cutoff in Hz the coefficients were computed for.
    cutoff: f32,
    /// `tan(π·fc/sr)` — the prewarped frequency coefficient.
    g: f32,
    /// `2/d`, `g/d`, `1/d` with `d = g·(g + 1/Q) + 1`.
    c0: f32,
    c1: f32,
    c2: f32,
    s0: f32,
    s1: f32,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            cutoff: UNTUNED,
            g: 0.0,
            c0: 0.0,
            c1: 0.0,
            c2: 0.0,
            s0: 0.0,
            s1: 0.0,
        }
    }
}

impl Mode {
    /// Recompute coefficients for `cutoff` Hz at Q `q`. Clamps match the Faust
    /// SVF (cutoff to `[1, 0.45·sr]`, Q to `[0.5, 30.5]`) so retuning is the
    /// only behavioral difference from running svf.dsp per sample. Cold path:
    /// runs once per (re)trigger per mode, not per sample.
    fn tune(&mut self, cutoff: f32, q: f32, sr: f32) {
        self.cutoff = cutoff;
        let g = (PI / sr * cutoff.clamp(1.0, 0.45 * sr)).tan();
        let d = g * (g + 1.0 / q.clamp(0.5, 30.5)) + 1.0;
        self.g = g;
        self.c0 = 2.0 / d;
        self.c1 = g / d;
        self.c2 = 1.0 / d;
    }

    /// One trapezoidal-SVF tick; returns the bandpass tap.
    #[inline]
    fn bandpass(&mut self, x: f32) -> f32 {
        let t1 = self.s0 + self.g * (x - self.s1);
        self.s0 = self.c0 * t1 - self.s0;
        let t2 = self.s1 + self.c1 * t1;
        self.s1 = 2.0 * t2 - self.s1;
        self.c2 * t1
    }
}

#[derive(Default)]
pub struct ModalBank {
    modes: [[Mode; MODAL_MODES]; CHANNELS],
    /// `(q, sr)` each channel's coefficients were last tuned at; `(0, 0)`
    /// after construction/reset never matches a real pair, forcing a retune.
    /// Per channel because the channels are processed by separate calls (with
    /// different detune offsets on `base`).
    tuned: [(f32, f32); CHANNELS],
}

impl ModalBank {
    /// Zero every resonator's state and drop the coefficient caches in place
    /// (plain-data overwrite, no heap alloc); the next `process` call retunes.
    pub fn reset_in_place(&mut self) {
        *self = Self::default();
    }

    /// One sample for channel `ch`: drive each mode's bandpass with `exc`, sum
    /// the per-mode outputs weighted by their decay envelope. `t` is seconds
    /// since the trigger. Modes whose center would land above `0.45·sr` are
    /// skipped (silent) rather than clamped — clamping would stack them on one
    /// frequency and sum to a resonant spike there. Output is normalized by
    /// the full mode count, so the level dips as modes fall off the top rather
    /// than the survivors getting louder.
    #[inline]
    pub fn process(&mut self, ch: usize, exc: f32, p: &ModalParams, t: f32, sr: f32) -> f32 {
        let nyq = sr * 0.45;
        let stale = self.tuned[ch] != (p.q, sr);
        self.tuned[ch] = (p.q, sr);

        // Running-product decay: env for mode m is decay(t, base_decay·(1+STEP·m)).
        let step = decay(t, p.base_decay * MODE_DECAY_STEP);
        let mut env = decay(t, p.base_decay);
        let mut acc = 0.0;
        for (mode, &ratio) in self.modes[ch].iter_mut().zip(p.ratios) {
            let cutoff = p.base * ratio;
            if cutoff <= nyq {
                if stale || cutoff != mode.cutoff {
                    mode.tune(cutoff, p.q, sr);
                }
                acc += mode.bandpass(exc) * env;
            } else {
                // Invalidate so a mode that later drops back below Nyquist
                // retunes even if `(q, sr)` changed while it was skipped.
                mode.cutoff = UNTUNED;
            }
            env *= step;
        }
        acc / MODAL_MODES as f32
    }
}
