mod convert;
pub mod manager;
mod receiver;
pub mod scope;
mod time;
pub mod types;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::Receiver;
use doux::Engine;

use receiver::SovaReceiver;
use time::TimeConverter;
use types::{AudioPayload, SyncTime};

pub use doux::audio;
pub use doux::config::DouxConfig;
pub use doux::error::DouxError;
pub use manager::{AudioEngineState, DouxManager};
pub use scope::ScopeCapture;

/// Creates a Sova integration for an existing engine.
///
/// This is the low-level API. For most use cases, prefer `DouxManager`
/// which handles the full engine lifecycle.
pub fn create_integration(
    engine: Arc<Mutex<Engine>>,
    rx: Receiver<AudioPayload>,
    initial_sync_time: SyncTime,
) -> JoinHandle<()> {
    let time_converter = TimeConverter::new(initial_sync_time);
    let receiver = SovaReceiver::new(engine, rx, time_converter);
    thread::spawn(move || receiver.run())
}
