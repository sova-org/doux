//! Low-level DSP primitives.

pub mod envelope;
pub mod fastmath;
pub mod fft;
pub mod filter;
pub mod noise;
pub mod oscillator;

pub use envelope::{init_envelope, Dahdsr, DahdsrState, EnvelopeParams};
pub use fastmath::{
    atan2f, cosf, exp2f, expf, expm1f, fast_tan, fast_tanh, fast_tanh_f32, ftz, log2f, modpi,
    par_cosf, par_sinf, pow10, pow1half, powf, sinf,
};
pub use filter::{Biquad, FilterState, Svf, SvfMode, SvfState};
pub use noise::{BrownNoise, PinkNoise};
pub use oscillator::{PhaseShape, Phasor};
