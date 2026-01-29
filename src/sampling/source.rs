//! Sample playback source from the lock-free registry.
//!
//! Native-only: provides real-time safe sample access via Arc<SampleData>.

use std::sync::Arc;

use super::cursor::Cursor;
use super::registry::SampleData;

/// Sample playback from the lock-free registry.
///
/// Holds an Arc to immutable sample data and a cursor for position tracking.
/// Safe to clone and use across threads.
pub struct RegistrySample {
    pub data: Arc<SampleData>,
    cursor: Cursor,
}

impl RegistrySample {
    /// Creates a new sample playback source.
    ///
    /// # Arguments
    /// * `data` - Arc to the sample data
    /// * `begin` - Start position as normalized value (0.0-1.0)
    /// * `end` - End position as normalized value (0.0-1.0)
    pub fn new(data: Arc<SampleData>, begin: f32, end: f32) -> Self {
        let cursor = Cursor::new(data.frame_count, begin, end);
        Self { data, cursor }
    }

    /// Updates the playback range.
    ///
    /// Pass `None` to keep the current value for either bound.
    pub fn update_range(&mut self, begin: Option<f32>, end: Option<f32>) {
        self.cursor.update_range(self.data.frame_count, begin, end);
    }

    /// Reads the sample value at current position with linear interpolation.
    #[inline]
    pub fn read(&self, channel: usize) -> f32 {
        self.data
            .read_interpolated(self.cursor.frame_position(), channel)
    }

    /// Advances the cursor by the given speed (frames per sample).
    #[inline]
    pub fn advance(&mut self, speed: f32) {
        self.cursor.advance(speed);
    }

    /// Returns true if playback has finished.
    #[inline]
    pub fn is_done(&self) -> bool {
        self.cursor.is_done()
    }
}

impl Clone for RegistrySample {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            cursor: self.cursor,
        }
    }
}
