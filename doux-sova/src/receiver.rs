use crossbeam_channel::{Receiver, Sender};
use doux::event::Event;
use sova_core::protocol::audio_engine_proxy::AudioEnginePayload;

use crate::manager::AudioCmd;
use crate::convert::payload_to_command;
use crate::time::TimeConverter;

pub struct SovaReceiver {
    cmd_tx: Sender<AudioCmd>,
    rx: Receiver<AudioEnginePayload>,
    time_converter: TimeConverter,
    sr: f64,
}

impl SovaReceiver {
    pub fn new(
        cmd_tx: Sender<AudioCmd>,
        rx: Receiver<AudioEnginePayload>,
        time_converter: TimeConverter,
        sr: f64,
    ) -> Self {
        Self {
            cmd_tx,
            rx,
            time_converter,
            sr,
        }
    }

    pub fn run(self) {
        while let Ok(payload) = self.rx.recv() {
            let cmd = payload_to_command(payload, &self.time_converter, self.sr);
            let event = Event::parse(&cmd, self.sr as f32);
            let _ = self.cmd_tx.send(AudioCmd::DispatchEvent(event));
        }
    }
}
