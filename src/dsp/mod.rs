//! Low-level DSP primitives.

pub mod envelope;
pub mod fastmath;
pub mod filter;
pub mod noise;
pub mod oscillator;

pub use envelope::{init_envelope, Adsr, AdsrState, EnvelopeParams};
pub use fastmath::{
    cosf, exp2f, expm1f, ftz, log2f, modpi, par_cosf, par_sinf, pow10, pow1half, powf, sinf,
};
pub use filter::{Biquad, FilterState};
pub use noise::{BrownNoise, PinkNoise};
pub use oscillator::{PhaseShape, Phasor};
