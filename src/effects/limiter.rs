//! Master bus peak limiter: linked across all output channels, instant
//! attack (zero overshoot, no lookahead), one-pole release. No user-facing
//! params — it only exists so the master sum stays out of the safety clip.

/// Gain reduction engages above this level; the tanh safety stage after the
/// limiter sees at most ~threshold and stays near-unity slope.
pub const LIMITER_THRESHOLD: f32 = 0.9;
pub const LIMITER_RELEASE_SECS: f32 = 0.1;

pub struct Limiter {
    env: f32,
    /// Smallest gain returned since the last `take_reduction`. Reporting only:
    /// hosts show it so the ceiling is visible instead of silently eating a mix.
    min_gain: f32,
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            env: 0.0,
            min_gain: 1.0,
        }
    }
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
        let gain = if self.env > LIMITER_THRESHOLD {
            LIMITER_THRESHOLD / self.env
        } else {
            1.0
        };
        if gain < self.min_gain {
            self.min_gain = gain;
        }
        gain
    }

    /// Peak gain reduction since the last call, as a 0..1 fraction (0 = the
    /// limiter never engaged). Reading resets the hold, so exactly one consumer
    /// may call it — the per-block metrics write.
    #[inline]
    pub fn take_reduction(&mut self) -> f32 {
        let reduction = 1.0 - self.min_gain;
        self.min_gain = 1.0;
        reduction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_is_unity_and_reports_no_reduction() {
        let mut lim = Limiter::default();
        for _ in 0..64 {
            assert_eq!(lim.process(0.5, 0.01), 1.0);
        }
        assert_eq!(lim.take_reduction(), 0.0);
    }

    #[test]
    fn above_threshold_holds_the_ceiling() {
        let mut lim = Limiter::default();
        let gain = lim.process(1.8, 0.01);
        // Instant attack: the very first over-threshold sample is already down.
        assert!((1.8 * gain - LIMITER_THRESHOLD).abs() < 1e-6, "gain {gain}");
    }

    #[test]
    fn reduction_reports_the_block_peak_then_resets() {
        let mut lim = Limiter::default();
        lim.process(0.5, 0.01); // unity
        lim.process(1.8, 0.01); // half gain
        lim.process(0.5, 0.01); // still releasing, so still reduced
        let reduction = lim.take_reduction();
        assert!(
            (reduction - 0.5).abs() < 1e-6,
            "peak reduction should come from the 1.8 sample, got {reduction}"
        );
        // Reading resets the hold; the envelope is still high, but the next
        // window reports only what happens in that window.
        assert!(lim.take_reduction() < reduction);
    }
}
