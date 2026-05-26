use std::sync::atomic::Ordering;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender, TrySendError};
use doux::event::Event;
use doux::telemetry::EngineMetrics;
use sova_core::protocol::audio_engine_proxy::AudioEnginePayload;

use crate::convert::payload_to_command;
use crate::manager::AudioCmd;
use crate::time::TimeConverter;

pub struct SovaReceiver {
    cmd_tx: Sender<AudioCmd>,
    rx: Receiver<AudioEnginePayload>,
    time_converter: TimeConverter,
    sr: f64,
    metrics: Arc<EngineMetrics>,
}

impl SovaReceiver {
    pub fn new(
        cmd_tx: Sender<AudioCmd>,
        rx: Receiver<AudioEnginePayload>,
        time_converter: TimeConverter,
        sr: f64,
        metrics: Arc<EngineMetrics>,
    ) -> Self {
        Self {
            cmd_tx,
            rx,
            time_converter,
            sr,
            metrics,
        }
    }

    pub fn run(self) {
        while let Ok(payload) = self.rx.recv() {
            let cmd = payload_to_command(payload, &self.time_converter, self.sr);
            let event = Event::parse(&cmd, self.sr as f32);
            match self.cmd_tx.try_send(AudioCmd::DispatchEvent(event)) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {
                    self.metrics.dropped_cmds.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Disconnected(_)) => {}
            }
        }
    }
}
