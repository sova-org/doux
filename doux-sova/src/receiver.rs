use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use doux::Engine;

use crate::convert::payload_to_command;
use crate::time::TimeConverter;
use crate::types::AudioPayload;

pub struct SovaReceiver {
    engine: Arc<Mutex<Engine>>,
    rx: Receiver<AudioPayload>,
    time_converter: TimeConverter,
}

impl SovaReceiver {
    pub fn new(
        engine: Arc<Mutex<Engine>>,
        rx: Receiver<AudioPayload>,
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
            let cmd = payload_to_command(&payload.args, payload.timetag, &self.time_converter);
            if let Ok(mut engine) = self.engine.lock() {
                engine.evaluate(&cmd);
            }
        }
    }
}
