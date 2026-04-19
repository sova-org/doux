//! Audio engine telemetry. Native only.

use serde::Serialize;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;

const LOAD_SCALE: f32 = 1_000_000.0; // fixed-point for atomic float storage
const DEFAULT_SMOOTHING: f32 = 0.6;
const PROFILE_PHASE_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfilePhase {
    BlockTotal,
    Schedule,
    SampleUpgrade,
    VoiceSource,
    VoiceFx,
    OrbitFx,
    FinalMix,
    RecorderCapture,
}

impl ProfilePhase {
    pub const ALL: [Self; PROFILE_PHASE_COUNT] = [
        Self::BlockTotal,
        Self::Schedule,
        Self::SampleUpgrade,
        Self::VoiceSource,
        Self::VoiceFx,
        Self::OrbitFx,
        Self::FinalMix,
        Self::RecorderCapture,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::BlockTotal => "block_total",
            Self::Schedule => "schedule",
            Self::SampleUpgrade => "sample_upgrade",
            Self::VoiceSource => "voice_source",
            Self::VoiceFx => "voice_fx",
            Self::OrbitFx => "orbit_fx",
            Self::FinalMix => "final_mix",
            Self::RecorderCapture => "recorder_capture",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::BlockTotal => 0,
            Self::Schedule => 1,
            Self::SampleUpgrade => 2,
            Self::VoiceSource => 3,
            Self::VoiceFx => 4,
            Self::OrbitFx => 5,
            Self::FinalMix => 6,
            Self::RecorderCapture => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PhaseProfile {
    pub total_ns: u64,
    pub calls: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ProfilingSnapshot {
    pub total_samples: u64,
    pub total_blocks: u64,
    pub phases: [PhaseProfile; PROFILE_PHASE_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhaseSummary {
    pub phase: ProfilePhase,
    pub label: &'static str,
    pub total_ns: u64,
    pub calls: u64,
    pub ns_per_sample: f64,
    pub percent_total: f64,
}

impl ProfilingSnapshot {
    pub fn merge_assign(&mut self, other: &Self) {
        self.total_samples += other.total_samples;
        self.total_blocks += other.total_blocks;
        for (dst, src) in self.phases.iter_mut().zip(other.phases.iter()) {
            dst.total_ns += src.total_ns;
            dst.calls += src.calls;
        }
    }

    pub fn phase(&self, phase: ProfilePhase) -> PhaseProfile {
        self.phases[phase.index()]
    }

    pub fn sorted_summaries(&self) -> Vec<PhaseSummary> {
        let total_block_ns = self.phase(ProfilePhase::BlockTotal).total_ns;
        let samples = self.total_samples.max(1) as f64;
        let mut summaries: Vec<_> = ProfilePhase::ALL
            .into_iter()
            .map(|phase| {
                let stat = self.phase(phase);
                let percent_total = if total_block_ns == 0 {
                    0.0
                } else {
                    stat.total_ns as f64 * 100.0 / total_block_ns as f64
                };
                PhaseSummary {
                    phase,
                    label: phase.label(),
                    total_ns: stat.total_ns,
                    calls: stat.calls,
                    ns_per_sample: stat.total_ns as f64 / samples,
                    percent_total,
                }
            })
            .filter(|summary| summary.total_ns > 0)
            .collect();

        summaries.sort_by(|a, b| {
            b.total_ns
                .cmp(&a.total_ns)
                .then_with(|| a.label.cmp(b.label))
        });
        summaries
    }
}

#[cfg(feature = "profiling")]
#[derive(Default)]
struct PhaseCounters {
    total_ns: AtomicU64,
    calls: AtomicU64,
}

/// Optional aggregate hotspot profiler for native engine development.
pub struct EngineProfiler {
    #[cfg(feature = "profiling")]
    total_samples: AtomicU64,
    #[cfg(feature = "profiling")]
    total_blocks: AtomicU64,
    #[cfg(feature = "profiling")]
    phases: [PhaseCounters; PROFILE_PHASE_COUNT],
}

impl Default for EngineProfiler {
    fn default() -> Self {
        Self {
            #[cfg(feature = "profiling")]
            total_samples: AtomicU64::new(0),
            #[cfg(feature = "profiling")]
            total_blocks: AtomicU64::new(0),
            #[cfg(feature = "profiling")]
            phases: std::array::from_fn(|_| PhaseCounters::default()),
        }
    }
}

impl EngineProfiler {
    pub fn record_phase(&self, phase: ProfilePhase, elapsed_ns: u64) {
        #[cfg(feature = "profiling")]
        {
            let counters = &self.phases[phase.index()];
            counters.total_ns.fetch_add(elapsed_ns, Ordering::Relaxed);
            counters.calls.fetch_add(1, Ordering::Relaxed);
        }
        #[cfg(not(feature = "profiling"))]
        {
            let _ = (phase, elapsed_ns);
        }
    }

    pub fn record_block(&self, samples: usize) {
        #[cfg(feature = "profiling")]
        {
            self.total_blocks.fetch_add(1, Ordering::Relaxed);
            self.total_samples
                .fetch_add(samples as u64, Ordering::Relaxed);
        }
        #[cfg(not(feature = "profiling"))]
        {
            let _ = samples;
        }
    }

    pub fn reset(&self) {
        #[cfg(feature = "profiling")]
        {
            self.total_samples.store(0, Ordering::Relaxed);
            self.total_blocks.store(0, Ordering::Relaxed);
            for counters in &self.phases {
                counters.total_ns.store(0, Ordering::Relaxed);
                counters.calls.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn snapshot(&self) -> ProfilingSnapshot {
        #[cfg(feature = "profiling")]
        {
            let mut snapshot = ProfilingSnapshot {
                total_samples: self.total_samples.load(Ordering::Relaxed),
                total_blocks: self.total_blocks.load(Ordering::Relaxed),
                phases: [PhaseProfile::default(); PROFILE_PHASE_COUNT],
            };
            for (idx, counters) in self.phases.iter().enumerate() {
                snapshot.phases[idx] = PhaseProfile {
                    total_ns: counters.total_ns.load(Ordering::Relaxed),
                    calls: counters.calls.load(Ordering::Relaxed),
                };
            }
            snapshot
        }
        #[cfg(not(feature = "profiling"))]
        {
            ProfilingSnapshot::default()
        }
    }
}

/// Measures DSP load as ratio of processing time to buffer time.
///
/// Thread-safe via atomics. Load of 1.0 means using all available time.
pub struct ProcessLoadMeasurer {
    buffer_time_ns: AtomicU64,
    load_fixed: AtomicU32,
    last_instant_fixed: AtomicU32,
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
            last_instant_fixed: AtomicU32::new(0),
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
        self.last_instant_fixed
            .store((instant_load * LOAD_SCALE) as u32, Ordering::Relaxed);

        let old_fixed = self.load_fixed.load(Ordering::Relaxed);
        let old_load = old_fixed as f32 / LOAD_SCALE;
        let new_load = self.smoothing * old_load + (1.0 - self.smoothing) * instant_load;
        let new_fixed = (new_load * LOAD_SCALE) as u32;

        self.load_fixed.store(new_fixed, Ordering::Relaxed);
    }

    pub fn get_load(&self) -> f32 {
        self.load_fixed.load(Ordering::Relaxed) as f32 / LOAD_SCALE
    }

    pub fn instant_load(&self) -> f32 {
        self.last_instant_fixed.load(Ordering::Relaxed) as f32 / LOAD_SCALE
    }

    pub fn reset(&self) {
        self.load_fixed.store(0, Ordering::Relaxed);
        self.last_instant_fixed.store(0, Ordering::Relaxed);
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
    pub profiler: EngineProfiler,
    pub active_voices: AtomicU32,
    pub peak_voices: AtomicU32,
    pub schedule_depth: AtomicU32,
    pub sample_pool_bytes: AtomicU64,
    pub time_bits: AtomicU64,
    pub dropped_events: AtomicU32,
}

impl Default for EngineMetrics {
    fn default() -> Self {
        Self {
            load: ProcessLoadMeasurer::default(),
            profiler: EngineProfiler::default(),
            active_voices: AtomicU32::new(0),
            peak_voices: AtomicU32::new(0),
            schedule_depth: AtomicU32::new(0),
            sample_pool_bytes: AtomicU64::new(0),
            time_bits: AtomicU64::new(0),
            dropped_events: AtomicU32::new(0),
        }
    }
}

impl EngineMetrics {
    pub fn reset_peak_voices(&self) {
        self.peak_voices.store(0, Ordering::Relaxed);
    }

    pub fn reset_profiling(&self) {
        self.profiler.reset();
    }

    pub fn profiling_snapshot(&self) -> ProfilingSnapshot {
        self.profiler.snapshot()
    }

    pub fn sample_pool_mb(&self) -> f32 {
        self.sample_pool_bytes.load(Ordering::Relaxed) as f32 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiling_snapshot_merges_totals() {
        let mut a = ProfilingSnapshot {
            total_samples: 100,
            total_blocks: 4,
            ..ProfilingSnapshot::default()
        };
        a.phases[ProfilePhase::VoiceSource.index()] = PhaseProfile {
            total_ns: 2_000,
            calls: 10,
        };

        let mut b = ProfilingSnapshot {
            total_samples: 50,
            total_blocks: 2,
            ..ProfilingSnapshot::default()
        };
        b.phases[ProfilePhase::VoiceSource.index()] = PhaseProfile {
            total_ns: 500,
            calls: 4,
        };

        a.merge_assign(&b);

        assert_eq!(a.total_samples, 150);
        assert_eq!(a.total_blocks, 6);
        assert_eq!(
            a.phase(ProfilePhase::VoiceSource),
            PhaseProfile {
                total_ns: 2_500,
                calls: 14,
            }
        );
    }

    #[test]
    fn summaries_sort_by_total_time() {
        let mut snapshot = ProfilingSnapshot {
            total_samples: 100,
            ..ProfilingSnapshot::default()
        };
        snapshot.phases[ProfilePhase::BlockTotal.index()] = PhaseProfile {
            total_ns: 10_000,
            calls: 1,
        };
        snapshot.phases[ProfilePhase::FinalMix.index()] = PhaseProfile {
            total_ns: 4_000,
            calls: 10,
        };
        snapshot.phases[ProfilePhase::VoiceSource.index()] = PhaseProfile {
            total_ns: 6_000,
            calls: 20,
        };

        let summaries = snapshot.sorted_summaries();

        assert_eq!(summaries[0].phase, ProfilePhase::BlockTotal);
        assert_eq!(summaries[1].phase, ProfilePhase::VoiceSource);
        assert_eq!(summaries[2].phase, ProfilePhase::FinalMix);
    }

    #[test]
    fn summaries_compute_percent_and_ns_per_sample() {
        let mut snapshot = ProfilingSnapshot {
            total_samples: 200,
            ..ProfilingSnapshot::default()
        };
        snapshot.phases[ProfilePhase::BlockTotal.index()] = PhaseProfile {
            total_ns: 20_000,
            calls: 2,
        };
        snapshot.phases[ProfilePhase::Schedule.index()] = PhaseProfile {
            total_ns: 5_000,
            calls: 20,
        };

        let schedule = snapshot
            .sorted_summaries()
            .into_iter()
            .find(|summary| summary.phase == ProfilePhase::Schedule)
            .unwrap();

        assert!((schedule.ns_per_sample - 25.0).abs() < f64::EPSILON);
        assert!((schedule.percent_total - 25.0).abs() < f64::EPSILON);
    }
}
