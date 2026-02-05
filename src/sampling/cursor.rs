//! Playback cursor for sample position tracking.
//!
//! Provides a reusable cursor that handles:
//! - Position tracking with fractional sample precision
//! - Begin/end range with automatic normalization
//! - Bidirectional playback (forward and reverse)
//! - Bounds checking for playback completion

/// Playback cursor for tracking position within a sample region.
///
/// The cursor operates in frame units (not normalized 0-1) for precision.
/// It supports bidirectional playback and automatically handles edge cases
/// like reversed begin/end values.
#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    /// Current position relative to start_pos (in frames).
    pos: f32,
    /// Absolute start position in frames.
    start_pos: f32,
    /// Length of the playback region in frames.
    length: f32,
    /// Whether playback has started (for reverse init).
    started: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: 0.0,
            start_pos: 0.0,
            length: 0.0,
            started: false,
        }
    }
}

impl Cursor {
    /// Creates a new cursor for a sample region.
    ///
    /// # Arguments
    /// * `frame_count` - Total frames in the sample
    /// * `begin` - Start position as normalized value (0.0-1.0)
    /// * `end` - End position as normalized value (0.0-1.0)
    ///
    /// Begin and end are automatically swapped if begin > end.
    pub fn new(frame_count: u32, begin: f32, end: f32) -> Self {
        let (start_pos, length) = Self::compute_range(frame_count, begin, end);
        Self {
            pos: 0.0,
            start_pos,
            length,
            started: false,
        }
    }

    /// Computes start position and length from normalized begin/end values.
    #[inline]
    fn compute_range(frame_count: u32, begin: f32, end: f32) -> (f32, f32) {
        let begin = begin.clamp(0.0, 1.0);
        let end = end.clamp(0.0, 1.0);
        let (lo, hi) = if begin <= end {
            (begin, end)
        } else {
            (end, begin)
        };
        let fc = frame_count as f32;
        (lo * fc, (hi - lo) * fc)
    }

    /// Updates the playback range while preserving relative position.
    ///
    /// Pass `None` to keep the current value for either bound.
    pub fn update_range(&mut self, frame_count: u32, begin: Option<f32>, end: Option<f32>) {
        let fc = frame_count as f32;
        let current_lo = self.start_pos / fc;
        let current_hi = current_lo + self.length / fc;
        let new_begin = begin.unwrap_or(current_lo);
        let new_end = end.unwrap_or(current_hi);
        let (start_pos, length) = Self::compute_range(frame_count, new_begin, new_end);
        self.start_pos = start_pos;
        self.length = length;
    }

    /// Recomputes the playback range for a new frame count, preserving position.
    pub fn upgrade_frame_count(&mut self, old_frame_count: u32, new_frame_count: u32) {
        let fc_old = old_frame_count as f32;
        let begin = self.start_pos / fc_old;
        let end = begin + self.length / fc_old;
        let (start_pos, length) = Self::compute_range(new_frame_count, begin, end);
        self.start_pos = start_pos;
        self.length = length;
    }

    /// Advances the cursor by the given speed (frames per sample).
    ///
    /// On first call, if speed is negative, position jumps to end for reverse playback.
    #[inline]
    pub fn advance(&mut self, speed: f32) {
        if !self.started {
            self.started = true;
            if speed < 0.0 {
                self.pos = self.length;
            }
        }
        self.pos += speed;
    }

    /// Returns true if playback has finished (position out of bounds).
    #[inline]
    pub fn is_done(&self) -> bool {
        self.pos < 0.0 || self.pos >= self.length
    }

    /// Returns the absolute frame position, clamped to valid range.
    #[inline]
    pub fn frame_position(&self) -> f32 {
        self.start_pos + self.pos.clamp(0.0, (self.length - 1.0).max(0.0))
    }

    /// Returns the fractional part of the current position for interpolation.
    #[inline]
    pub fn frac(&self) -> f32 {
        self.pos.clamp(0.0, (self.length - 1.0).max(0.0)).fract()
    }

    /// Returns the current frame index (integer part of position).
    #[inline]
    pub fn current_frame(&self) -> usize {
        (self.start_pos + self.pos.clamp(0.0, (self.length - 1.0).max(0.0))) as usize
    }

    /// Returns the next frame index for interpolation, clamped to end boundary.
    #[inline]
    pub fn next_frame(&self, frame_count: u32) -> usize {
        let current = self.current_frame();
        let end_frame = (self.start_pos + self.length) as usize;
        if current + 1 < end_frame {
            current + 1
        } else {
            current.min(frame_count as usize - 1)
        }
    }

    /// Returns the playback region length in frames.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_full_range() {
        let c = Cursor::new(1000, 0.0, 1.0);
        assert_eq!(c.start_pos, 0.0);
        assert_eq!(c.length, 1000.0);
        assert_eq!(c.pos, 0.0);
        assert!(!c.started);
    }

    #[test]
    fn new_partial_range() {
        let c = Cursor::new(1000, 0.25, 0.75);
        assert_eq!(c.start_pos, 250.0);
        assert_eq!(c.length, 500.0);
    }

    #[test]
    fn new_reversed_range_normalizes() {
        let c = Cursor::new(1000, 0.75, 0.25);
        assert_eq!(c.start_pos, 250.0);
        assert_eq!(c.length, 500.0);
    }

    #[test]
    fn new_clamps_out_of_bounds() {
        let c = Cursor::new(1000, -0.5, 1.5);
        assert_eq!(c.start_pos, 0.0);
        assert_eq!(c.length, 1000.0);
    }

    #[test]
    fn advance_forward() {
        let mut c = Cursor::new(1000, 0.0, 1.0);
        c.advance(1.0);
        assert!(c.started);
        assert_eq!(c.pos, 1.0);
        c.advance(1.0);
        assert_eq!(c.pos, 2.0);
    }

    #[test]
    fn advance_reverse_starts_at_end() {
        let mut c = Cursor::new(1000, 0.0, 1.0);
        c.advance(-1.0);
        assert!(c.started);
        assert_eq!(c.pos, 999.0); // length - 1
    }

    #[test]
    fn is_done_forward() {
        let mut c = Cursor::new(100, 0.0, 1.0);
        assert!(!c.is_done());
        c.pos = 99.0;
        assert!(!c.is_done());
        c.pos = 100.0;
        assert!(c.is_done());
    }

    #[test]
    fn is_done_reverse() {
        let mut c = Cursor::new(100, 0.0, 1.0);
        c.pos = 0.0;
        assert!(!c.is_done());
        c.pos = -0.1;
        assert!(c.is_done());
    }

    #[test]
    fn frame_position_clamped() {
        let mut c = Cursor::new(1000, 0.25, 0.75);
        c.pos = -10.0;
        assert_eq!(c.frame_position(), 250.0);
        c.pos = 600.0;
        assert_eq!(c.frame_position(), 749.0); // 250 + 499 (length-1)
    }

    #[test]
    fn update_range_partial() {
        let mut c = Cursor::new(1000, 0.0, 1.0);
        c.update_range(1000, Some(0.1), None);
        assert_eq!(c.start_pos, 100.0);
        assert_eq!(c.length, 900.0);
    }

    #[test]
    fn update_range_both() {
        let mut c = Cursor::new(1000, 0.0, 1.0);
        c.update_range(1000, Some(0.2), Some(0.8));
        assert_eq!(c.start_pos, 200.0);
        assert_eq!(c.length, 600.0);
    }

    #[test]
    fn current_and_next_frame() {
        let mut c = Cursor::new(1000, 0.0, 1.0);
        c.pos = 5.5;
        assert_eq!(c.current_frame(), 5);
        assert_eq!(c.next_frame(1000), 6);
        assert!((c.frac() - 0.5).abs() < 0.001);
    }

    #[test]
    fn next_frame_at_boundary() {
        let mut c = Cursor::new(1000, 0.0, 0.1); // length = 100
        c.pos = 99.0;
        assert_eq!(c.current_frame(), 99);
        assert_eq!(c.next_frame(1000), 99); // clamped, can't go past end
    }
}
