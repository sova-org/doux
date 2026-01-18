//! Sova event receiver thread.
//!
//! Listens for events from Sova's scheduler via a crossbeam channel and
//! forwards them to the Doux engine as command strings.

use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use doux::Engine;
use sova_core::protocol::audio_engine_proxy::AudioEnginePayload;

use crate::convert::payload_to_command;
use crate::time::TimeConverter;

/// Receives events from Sova and forwards them to the Doux engine.
///
/// Runs in a dedicated thread, blocking on channel receive. Exits when
/// the sender is dropped (channel closed).
pub struct SovaReceiver {
    /// Shared reference to the audio engine.
    engine: Arc<Mutex<Engine>>,
    /// Channel receiving events from Sova's scheduler.
    rx: Receiver<AudioEnginePayload>,
    /// Converts Sova timestamps to engine time.
    time_converter: TimeConverter,
}

impl SovaReceiver {
    /// Creates a new receiver with the given engine, channel, and time converter.
    pub fn new(
        engine: Arc<Mutex<Engine>>,
        rx: Receiver<AudioEnginePayload>,
        time_converter: TimeConverter,
    ) -> Self {
        Self {
            engine,
            rx,
            time_converter,
        }
    }

    /// Runs the receiver loop until the channel is closed.
    ///
    /// Each received payload is converted to a Doux command string
    /// and evaluated by the engine.
    pub fn run(self) {
        while let Ok(payload) = self.rx.recv() {
            let cmd = payload_to_command(&payload.args, payload.timetag, &self.time_converter);
            if let Ok(mut engine) = self.engine.lock() {
                engine.evaluate(&cmd);
            }
        }
    }
}
