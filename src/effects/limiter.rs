//! Master bus peak limiter: linked across all output channels, instant
//! attack (zero overshoot, no lookahead), one-pole release. No user-facing
//! params — it only exists so the master sum stays out of the safety clip.

/// Gain reduction engages above this level; the tanh safety stage after the
/// limiter sees at most ~threshold and stays near-unity slope.
pub const LIMITER_THRESHOLD: f32 = 0.9;
pub const LIMITER_RELEASE_SECS: f32 = 0.1;

#[derive(Default)]
pub struct Limiter {
    env: f32,
}

impl Limiter {
    /// Feed this frame's linked peak (max |sample| across all channels);
    /// returns the gain to apply to every channel of the frame.
    #[inline]
    pub fn process(&mut self, peak: f32, release_coeff: f32) -> f32 {
        if peak > self.env {
            self.env = peak; // instant attack: no overshoot past threshold
        } else {
            self.env += release_coeff * (peak - self.env);
        }
        if self.env > LIMITER_THRESHOLD {
            LIMITER_THRESHOLD / self.env
        } else {
            1.0
        }
    }
}
