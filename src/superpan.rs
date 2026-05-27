//! Multichannel `superpan` — SuperCollider `PanAz`-style equal-power azimuth
//! panning over an ordered set of output pairs.
//!
//! A voice's stereo signal is rung around output PAIRS by [`panaz_gains`] (one
//! ring node per pair, L/R preserved into each pair's two channels);
//! [`SpeakerSet`] selects *which* output pairs form that ring (e.g. a contiguous
//! front block `1,2,3,4` or the odd pairs `1,3,5,7`).

use std::f32::consts::PI;

/// Upper bound on ring nodes (output pairs) a single `superpan` voice can span.
/// Bounds the fixed-size gain and pair-set arrays so the audio path never allocates.
pub const MAX_SUPERPAN_NODES: usize = 64;

/// Equal-power azimuth pan over a ring of `num_nodes` evenly-spaced nodes.
///
/// `pos` is the ring position (wraps 0..1); `width` is how many adjacent nodes
/// are lit (~2 = a localised pair, larger spreads the source wider). Writes the
/// first `num_nodes` entries of `gains_out`; gains are normalised so
/// `Σ gain² == 1`, keeping loudness constant as the source moves. `num_nodes <= 1`
/// yields a single unity gain.
#[inline]
pub fn panaz_gains(
    num_nodes: usize,
    pos: f32,
    width: f32,
    gains_out: &mut [f32; MAX_SUPERPAN_NODES],
) {
    let n = num_nodes.min(MAX_SUPERPAN_NODES);
    for g in gains_out.iter_mut().take(n) {
        *g = 0.0;
    }
    if n <= 1 {
        if n == 1 {
            gains_out[0] = 1.0;
        }
        return;
    }
    let nf = n as f32;
    let half_w = width.max(1e-4) * 0.5;
    let pos = pos - pos.floor(); // wrap into [0,1)
    let center = pos * nf; // position in node units
    for (i, g) in gains_out.iter_mut().take(n).enumerate() {
        // Shortest distance around the ring between node `i` and the centre.
        let mut d = i as f32 - center;
        d -= (d / nf).round() * nf;
        let d = d.abs();
        if d < half_w {
            *g = ((d / half_w) * (PI * 0.5)).cos();
        }
    }
    let energy: f32 = gains_out.iter().take(n).map(|g| g * g).sum();
    if energy > 1e-12 {
        let inv = 1.0 / energy.sqrt();
        for g in gains_out.iter_mut().take(n) {
            *g *= inv;
        }
    }
}

/// Ordered, fixed-capacity selection of output PAIRS a `superpan` voice spans
/// (pair `p` = channels `2p`/`2p+1`). Empty means "all pairs, in order".
#[derive(Clone, Copy, Debug)]
pub struct SpeakerSet {
    pairs: [u16; MAX_SUPERPAN_NODES],
    len: u8,
}

impl SpeakerSet {
    pub const fn empty() -> Self {
        Self {
            pairs: [0; MAX_SUPERPAN_NODES],
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Output-pair index at ring position `k` (0-based). Caller guarantees
    /// `k < len`.
    pub fn get(&self, k: usize) -> usize {
        self.pairs[k] as usize
    }

    /// Parse a comma-separated, 1-based output-pair list (e.g. `"1,3,5,7"`) into
    /// a 0-based set. Returns `None` for an empty or malformed list, or one
    /// containing pair `0` (pairs are 1-based in the event string).
    /// Entries beyond [`MAX_SUPERPAN_NODES`] are dropped.
    pub fn parse(s: &str) -> Option<Self> {
        let mut set = Self::empty();
        for tok in s.split(',') {
            let one_based: usize = tok.parse().ok()?;
            if one_based == 0 {
                return None;
            }
            if (set.len as usize) < MAX_SUPERPAN_NODES {
                set.pairs[set.len as usize] = (one_based - 1) as u16;
                set.len += 1;
            }
        }
        if set.len == 0 {
            None
        } else {
            Some(set)
        }
    }
}
