//! Time-based event scheduling with sorted storage.
//!
//! Manages a queue of [`Event`]s that should fire at specific times.
//! Events are kept sorted by time for O(1) early-exit when no events are ready.
//!
//! # Event Lifecycle
//!
//! 1. Event with `time` field is parsed → inserted in sorted order
//! 2. Engine calls `process_schedule()` each sample
//! 3. When `event.time <= engine.time`:
//!    - Event fires (triggers voice/sound)
//!    - If `repeat` is set, event is re-inserted with new time
//!    - Otherwise, event is removed
//!
//! # Complexity
//!
//! - Insertion: O(N) due to sorted insert (infrequent, ~10-100/sec)
//! - Processing: O(1) when no events ready (99.9% of calls)
//! - Processing: O(K) when K events fire (rare)
//!
//! # Capacity
//!
//! Limited to [`MAX_EVENTS`](crate::types::MAX_EVENTS) to prevent unbounded
//! growth. Events beyond this limit are silently dropped.

use crate::event::Event;
use crate::types::MAX_EVENTS;

/// Queue of time-scheduled events, sorted by time ascending.
///
/// Invariant: `events[i].time <= events[i+1].time` for all valid indices.
/// This enables O(1) early-exit: if `events[0].time > now`, no events are ready.
pub struct Schedule {
    events: Vec<Event>,
}

impl Schedule {
    /// Creates an empty schedule with pre-allocated capacity.
    pub fn new() -> Self {
        Self {
            events: Vec::with_capacity(MAX_EVENTS),
        }
    }

    /// Adds an event to the schedule in sorted order.
    ///
    /// Events at capacity are silently dropped.
    /// Insertion is O(N) but occurs infrequently (user actions).
    pub fn push(&mut self, event: Event) {
        if self.events.len() >= MAX_EVENTS {
            return;
        }
        let time = event.time.unwrap_or(f64::MAX);
        let pos = self
            .events
            .partition_point(|e| e.time.unwrap_or(f64::MAX) < time);
        self.events.insert(pos, event);
    }

    /// Returns the time of the earliest event, if any.
    #[inline]
    pub fn peek_time(&self) -> Option<f64> {
        self.events.first().and_then(|e| e.time)
    }

    /// Removes and returns the earliest event.
    #[inline]
    pub fn pop_front(&mut self) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    /// Returns the number of scheduled events.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if no events are scheduled.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Removes all scheduled events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}
