//! Distortion-class effects.
//!
//! The saturator (`distort`), wavefolder (`fold`) and phase wrapper (`wrap`)
//! are now Faust-generated and live in [`super::faust_dsp`]. What remains here:
//!
//! - [`DcBlocker`]: single-pole DC-removal HPF (~20 Hz corner), run after the
//!   distortion stages to remove the DC their baked-in bias introduces.

use crate::types::{ModuleGroup, ModuleInfo, ParamInfo, StereoFrame};

pub const INFO: ModuleInfo = ModuleInfo {
    name: "distort",
    description: "Waveshaping distortion (saturation, wavefolding, phase wrapping)",
    group: ModuleGroup::Effect,
    params: &[
        ParamInfo {
            name: "distort",
            aliases: &[],
            description:
                "soft saturation amount (unbounded — saturates to hard clip at high values)",
            default: "0.0",
            min: 0.0,
            max: f32::MAX,
        },
        ParamInfo {
            name: "fold",
            aliases: &[],
            description: "wavefolding amount",
            default: "0.0",
            min: 0.0,
            max: 1.0,
        },
        ParamInfo {
            name: "foldmode",
            aliases: &["fmode"],
            description: "fold shape (triangle, sine, wrap)",
            default: "triangle",
            min: 0.0,
            max: 2.0,
        },
        ParamInfo {
            name: "wrap",
            aliases: &[],
            description: "phase wrapping amount",
            default: "0.0",
            min: 0.0,
            max: 10.0,
        },
        ParamInfo {
            name: "distortvol",
            aliases: &[],
            description: "output volume compensation",
            default: "1.0",
            min: 0.0,
            max: 2.0,
        },
        ParamInfo {
            name: "distortmode",
            aliases: &["dmode"],
            description: "saturator curve (soft, tanh, arctan, hardclip, parabolic, sinarctan)",
            default: "soft",
            min: 0.0,
            max: 5.0,
        },
        ParamInfo {
            name: "distortasym",
            aliases: &["dasym"],
            description: "pre-shaper bias for asymmetric / even-harmonic drive",
            default: "0.0",
            min: -1.0,
            max: 1.0,
        },
    ],
};

/// First-order DC blocker. `y = x − x₋₁ + R · y₋₁` with `R = 0.9995`
/// (≈ 20 Hz corner at 48 kHz). Cheap; removes the DC creep introduced by
/// asymmetric drive + modulation upstream.
#[derive(Clone, Copy, Default)]
pub struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
}

impl DcBlocker {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        const R: f32 = 0.9995;
        let y = x - self.x_prev + R * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }

    #[inline]
    pub fn process_block(&mut self, buf: &mut [StereoFrame], n: usize, ch: usize) {
        const R: f32 = 0.9995;
        for slot in buf.iter_mut().take(n) {
            let x = slot[ch];
            let y = x - self.x_prev + R * self.y_prev;
            self.x_prev = x;
            self.y_prev = y;
            slot[ch] = y;
        }
    }
}
