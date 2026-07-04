//! Device-boundary speaker protection.
//!
//! The DSP core is deliberately IEEE-transparent (a stateless NaN/inf is documented behavior,
//! see e.g. `/` and `linlin`), but a NaN must never reach the DAC. This is the one guard between
//! the engine and the speakers: the realtime shells (`audio.rs`, `wasm.rs`) call it after the host
//! renders. The offline renderer and the harness read the engine raw, so the bit-exact corpus is
//! untouched (the same category as the fade mix).

use std::sync::atomic::{AtomicU32, Ordering};

/// Running total of non-finite output samples zapped at the device boundary (see
/// [`sanitize_block`]). Same diagnostic pattern as the skipped-migration counter: the audio thread does
/// one relaxed add per affected block, the control thread reads it off the realtime path. It
/// stays at zero unless a program computes a NaN/inf (e.g. a zero-span `linlin`).
static NONFINITE_ZAPPED: AtomicU32 = AtomicU32::new(0);

/// The running total of non-finite output samples zapped at the device boundary.
pub fn nonfinite_zapped() -> u32 {
    NONFINITE_ZAPPED.load(Ordering::Relaxed)
}

/// Replace non-finite samples with 0 in a device-bound block, counting what was zapped.
///
/// Speaker protection for the live instrument: the engine's DSP is deliberately
/// IEEE-transparent (a stateless NaN/inf is documented behavior, see e.g. `/` and `linlin`), but
/// a NaN must never reach the DAC. This lives at the *device* boundary — the realtime shells
/// (`audio.rs`, `wasm.rs`) call it after the host renders; the offline renderer and the harness
/// read the engine raw, so the bit-exact corpus is untouched (same category as the fade mix).
/// Realtime-safe: a bounded pass, one relaxed atomic add only when something was zapped.
pub fn sanitize_block(block: &mut [f32]) {
    let mut zapped = 0_u32;
    for s in block.iter_mut() {
        if !s.is_finite() {
            *s = 0.0;
            zapped += 1;
        }
    }
    if zapped > 0 {
        NONFINITE_ZAPPED.fetch_add(zapped, Ordering::Relaxed);
    }
}
