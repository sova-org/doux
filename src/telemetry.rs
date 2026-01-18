//! Audio engine telemetry. Native only.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

const LOAD_SCALE: f32 = 1_000_000.0; // fixed-point for atomic float storage
const DEFAULT_SMOOTHING: f32 = 0.9;

/// Measures DSP load as ratio of processing time to buffer time.
///
/// Thread-safe via atomics. Load of 1.0 means using all available time.
pub struct ProcessLoadMeasurer {
    buffer_time_ns: AtomicU64,
    load_fixed: AtomicU32,
    smoothing: f32,
}

impl Default for ProcessLoadMeasurer {
    fn default() -> Self {
        Self::new(DEFAULT_SMOOTHING)
    }
}

impl ProcessLoadMeasurer {
    /// Creates a new measurer. Smoothing in [0.0, 0.99]: higher = slower response.
    pub fn new(smoothing: f32) -> Self {
        Self {
            buffer_time_ns: AtomicU64::new(0),
            load_fixed: AtomicU32::new(0),
            smoothing: smoothing.clamp(0.0, 0.99),
        }
    }

    pub fn set_buffer_time(&self, ns: u64) {
        self.buffer_time_ns.store(ns, Ordering::Relaxed);
    }

    /// Returns a timer that records elapsed time on drop.
    pub fn start_timer(&self) -> ScopedTimer<'_> {
        ScopedTimer {
            measurer: self,
            start: Instant::now(),
        }
    }

    pub fn record_sample(&self, elapsed_ns: u64) {
        let buffer_ns = self.buffer_time_ns.load(Ordering::Relaxed);
        if buffer_ns == 0 {
            return;
        }

        let instant_load = (elapsed_ns as f64 / buffer_ns as f64).min(2.0) as f32;
        let old_fixed = self.load_fixed.load(Ordering::Relaxed);
        let old_load = old_fixed as f32 / LOAD_SCALE;
        let new_load = self.smoothing * old_load + (1.0 - self.smoothing) * instant_load;
        let new_fixed = (new_load * LOAD_SCALE) as u32;

        self.load_fixed.store(new_fixed, Ordering::Relaxed);
    }

    pub fn get_load(&self) -> f32 {
        self.load_fixed.load(Ordering::Relaxed) as f32 / LOAD_SCALE
    }

    pub fn reset(&self) {
        self.load_fixed.store(0, Ordering::Relaxed);
    }
}

/// RAII timer that records elapsed time on drop.
pub struct ScopedTimer<'a> {
    measurer: &'a ProcessLoadMeasurer,
    start: Instant,
}

impl Drop for ScopedTimer<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_nanos() as u64;
        self.measurer.record_sample(elapsed);
    }
}

/// Aggregated engine metrics. All fields atomic for cross-thread access.
pub struct EngineMetrics {
    pub load: ProcessLoadMeasurer,
    pub active_voices: AtomicU32,
    pub peak_voices: AtomicU32,
    pub schedule_depth: AtomicU32,
    pub sample_pool_bytes: AtomicU64,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self {
            load: ProcessLoadMeasurer::default(),
            active_voices: AtomicU32::new(0),
            peak_voices: AtomicU32::new(0),
            schedule_depth: AtomicU32::new(0),
            sample_pool_bytes: AtomicU64::new(0),
        }
    }
}

impl EngineMetrics {
    pub fn reset_peak_voices(&self) {
        self.peak_voices.store(0, Ordering::Relaxed);
    }

    pub fn sample_pool_mb(&self) -> f32 {
        self.sample_pool_bytes.load(Ordering::Relaxed) as f32 / (1024.0 * 1024.0)
    }
}
