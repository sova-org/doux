use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use doux::Engine;
use sova_core::protocol::audio_engine_proxy::AudioEnginePayload;

use crate::convert::payload_to_command;
use crate::time::TimeConverter;

pub struct SovaReceiver {
    engine: Arc<Mutex<Engine>>,
    rx: Receiver<AudioEnginePayload>,
    time_converter: TimeConverter,
}

impl SovaReceiver {
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

    pub fn run(self) {
        while let Ok(payload) = self.rx.recv() {
            let cmd = payload_to_command(payload, &self.time_converter);
            if let Ok(mut engine) = self.engine.lock() {
                engine.evaluate(&cmd);
            }
        }
    }
}
