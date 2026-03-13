//! Tick-based event scheduling with sorted storage.
//!
//! Manages a queue of [`Event`]s that should fire at specific sample ticks.
//! Events are kept sorted by tick for O(1) early-exit when no events are ready.
//!
//! # Event Lifecycle
//!
//! 1. Event with `tick` field is parsed → inserted in sorted order
//! 2. Engine calls `process_schedule()` each sample
//! 3. When `event.tick <= engine.tick`:
//!    - Event fires (triggers voice/sound)
//!    - Event is removed
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

use std::collections::VecDeque;

use crate::event::Event;
use crate::types::MAX_EVENTS;

/// Queue of tick-scheduled events, sorted by tick ascending.
///
/// Invariant: `events[i].tick <= events[i+1].tick` for all valid indices.
/// This enables O(1) early-exit: if `events[0].tick > now`, no events are ready.
pub struct Schedule {
    events: VecDeque<Event>,
}

impl Schedule {
    /// Creates an empty schedule with pre-allocated capacity.
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_EVENTS),
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
        let tick = event.tick.unwrap_or(u64::MAX);
        let pos = self
            .events
            .make_contiguous()
            .partition_point(|e| e.tick.unwrap_or(u64::MAX) < tick);
        self.events.insert(pos, event);
    }

    /// Returns the tick of the earliest event, if any.
    #[inline]
    pub fn peek_tick(&self) -> Option<u64> {
        self.events.front().and_then(|e| e.tick)
    }

    /// Removes and returns the earliest event.
    #[inline]
    pub fn pop_front(&mut self) -> Option<Event> {
        self.events.pop_front()
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
