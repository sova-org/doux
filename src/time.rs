//! Wall-clock → engine-tick conversion.
//!
//! Used by the OSC receiver to honor OSC bundle timetags so external clients
//! (Tidal, Zwirn, …) get sample-accurate scheduling on the same footing as
//! the in-process Sova/Cagire path.

/// Seconds between the NTP epoch (1900-01-01) and the Unix epoch (1970-01-01).
const NTP_UNIX_OFFSET_SECS: u64 = 2_208_988_800;

/// Maps wall-clock microseconds to engine sample ticks.
///
/// Captured once at engine boot and cloned into the OSC receiver thread.
#[derive(Clone, Copy, Debug)]
pub struct TimeAnchor {
    pub start_unix_micros: u64,
    pub sample_rate: f32,
}

impl TimeAnchor {
    pub fn unix_micros_to_tick(&self, micros: u64) -> u64 {
        let delta = micros.saturating_sub(self.start_unix_micros);
        ((delta as f64 * self.sample_rate as f64) / 1_000_000.0).round() as u64
    }

    /// Resolve an OSC NTP timetag to an engine tick.
    ///
    /// Returns `None` for the OSC "immediately" sentinel `(0, 1)` and for
    /// timetags preceding the Unix epoch — both cases mean "fire on receipt".
    pub fn ntp_to_tick(&self, secs: u32, fractional: u32) -> Option<u64> {
        if secs == 0 && fractional == 1 {
            return None;
        }
        let secs_ntp = secs as u64;
        if secs_ntp < NTP_UNIX_OFFSET_SECS {
            return None;
        }
        let secs_unix = secs_ntp - NTP_UNIX_OFFSET_SECS;
        let frac_micros = ((fractional as u64) * 1_000_000) >> 32;
        let unix_micros = secs_unix.checked_mul(1_000_000)?.checked_add(frac_micros)?;
        Some(self.unix_micros_to_tick(unix_micros))
    }
}
