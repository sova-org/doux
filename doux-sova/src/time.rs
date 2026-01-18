//! Time synchronization between Sova and Doux.
//!
//! Sova uses microsecond timestamps (SyncTime) from its internal clock.
//! Doux uses seconds relative to engine start. This module converts between them.

use sova_core::clock::SyncTime;

/// Converts Sova timestamps to Doux engine time.
///
/// Stores the initial sync time (engine start) and computes relative
/// offsets in seconds for incoming events.
pub struct TimeConverter {
    /// Microsecond timestamp when the engine was started.
    engine_start_micros: SyncTime,
}

impl TimeConverter {
    /// Creates a converter with the given initial sync time.
    ///
    /// Pass `clock.micros()` at engine startup.
    pub fn new(initial_sync_time: SyncTime) -> Self {
        Self {
            engine_start_micros: initial_sync_time,
        }
    }

    /// Converts a Sova timetag to engine time in seconds.
    ///
    /// Returns the number of seconds since engine start.
    pub fn sync_to_engine_time(&self, timetag: SyncTime) -> f64 {
        let delta = timetag.saturating_sub(self.engine_start_micros);
        (delta as f64) / 1_000_000.0
    }
}
