//! Colored noise generators.
//!
//! Transforms white noise into spectrally-shaped noise with different frequency
//! characteristics. Both generators are stateful filters that process white noise
//! sample-by-sample.
//!
//! # Noise Colors
//!
//! | Color | Slope      | Character                        |
//! |-------|------------|----------------------------------|
//! | White | 0 dB/oct   | Equal energy per frequency       |
//! | Pink  | -3 dB/oct  | Equal energy per octave          |
//! | Brown | -6 dB/oct  | Rumbling, emphasizes low freqs   |

/// Pink noise generator using the Voss-McCartney algorithm.
///
/// Applies a parallel bank of first-order lowpass filters to shape white noise
/// into pink noise with -3 dB/octave rolloff. The coefficients approximate an
/// ideal pink spectrum across the audio range.
///
/// Also known as 1/f noise, pink noise has equal energy per octave, making it
/// useful for audio testing and as a natural-sounding noise source.
#[derive(Clone, Copy)]
pub struct PinkNoise {
    b: [f32; 7],
}

impl Default for PinkNoise {
    fn default() -> Self {
        Self { b: [0.0; 7] }
    }
}

impl PinkNoise {
    /// Processes one white noise sample and returns the corresponding pink noise sample.
    ///
    /// The input should be uniformly distributed white noise in the range `[-1, 1]`.
    /// Output is scaled to approximately the same amplitude range.
    pub fn next(&mut self, white: f32) -> f32 {
        self.b[0] = 0.99886 * self.b[0] + white * 0.0555179;
        self.b[1] = 0.99332 * self.b[1] + white * 0.0750759;
        self.b[2] = 0.96900 * self.b[2] + white * 0.153852;
        self.b[3] = 0.86650 * self.b[3] + white * 0.3104856;
        self.b[4] = 0.55000 * self.b[4] + white * 0.5329522;
        self.b[5] = -0.7616 * self.b[5] - white * 0.0168980;
        let pink = self.b[0]
            + self.b[1]
            + self.b[2]
            + self.b[3]
            + self.b[4]
            + self.b[5]
            + self.b[6]
            + white * 0.5362;
        self.b[6] = white * 0.115926;
        pink * 0.11
    }
}

/// Brown noise generator using leaky integration.
///
/// Applies a simple first-order lowpass filter (leaky integrator) to produce
/// noise with -6 dB/octave rolloff. Named after Robert Brown (Brownian motion),
/// not the color.
///
/// Also known as red noise or random walk noise. Has a deep, rumbling character
/// with strong low-frequency content.
#[derive(Clone, Copy)]
pub struct BrownNoise {
    out: f32,
}

impl Default for BrownNoise {
    fn default() -> Self {
        Self { out: 0.0 }
    }
}

impl BrownNoise {
    /// Processes one white noise sample and returns the corresponding brown noise sample.
    ///
    /// The input should be uniformly distributed white noise in the range `[-1, 1]`.
    /// Output amplitude depends on the integration coefficient.
    pub fn next(&mut self, white: f32) -> f32 {
        self.out = (self.out + 0.02 * white) / 1.02;
        self.out
    }
}
