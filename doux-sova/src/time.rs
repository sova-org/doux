use sova_core::clock::SyncTime;

pub struct TimeConverter {
    engine_start_micros: SyncTime,
}

impl TimeConverter {
    pub fn new(initial_sync_time: SyncTime) -> Self {
        Self {
            engine_start_micros: initial_sync_time,
        }
    }

    pub fn sync_to_engine_tick(&self, timetag: SyncTime, sr: f64) -> u64 {
        let delta = timetag.saturating_sub(self.engine_start_micros);
        ((delta as f64 * sr) / 1_000_000.0).round() as u64
    }
}
