mod convert;
pub mod manager;
mod receiver;
pub mod scope;
mod time;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::unbounded;
use doux::Engine;
use sova_core::clock::SyncTime;
use sova_core::protocol::audio_engine_proxy::AudioEngineProxy;

use receiver::SovaReceiver;
use time::TimeConverter;

// Re-exports for convenience
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
    initial_sync_time: SyncTime,
) -> (JoinHandle<()>, AudioEngineProxy) {
    let (tx, rx) = unbounded();
    let time_converter = TimeConverter::new(initial_sync_time);
    let receiver = SovaReceiver::new(engine, rx, time_converter);
    let handle = thread::spawn(move || receiver.run());
    (handle, AudioEngineProxy::new(tx))
}
