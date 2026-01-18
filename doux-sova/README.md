# doux-sova

Integration layer connecting Doux audio engine to Sova's AudioEngineProxy.

## Quick Start with DouxManager

```rust
use doux_sova::{DouxManager, DouxConfig, audio};

// 1. List available devices (for UI)
let devices = audio::list_output_devices();
for dev in &devices {
    let default_marker = if dev.is_default { " [default]" } else { "" };
    println!("{}: {} (max {} ch){}", dev.index, dev.name, dev.max_channels, default_marker);
}

// 2. Create configuration
let config = DouxConfig::default()
    .with_output_device("Built-in Output")
    .with_channels(2)
    .with_buffer_size(512)
    .with_sample_path("/path/to/samples");

// 3. Create and start manager
let mut manager = DouxManager::new(config)?;
let proxy = manager.start(clock.micros())?;

// 4. Register with Sova
device_map.connect_audio_engine("doux", proxy)?;

// 5. Control during runtime
manager.hush();                // Release all voices
manager.panic();               // Immediately stop all voices
manager.add_sample_path(path); // Add more samples
manager.rescan_samples();      // Rescan all sample directories
let state = manager.state();   // Get AudioEngineState (voices, cpu, etc)
let scope = manager.scope_capture(); // Get oscilloscope buffer

// 6. Stop or restart
manager.stop();              // Stop audio streams
let new_config = DouxConfig::default().with_channels(4);
let new_proxy = manager.restart(new_config, clock.micros())?;
```

## Low-Level API

For direct engine access without lifecycle management:

```rust
use std::sync::{Arc, Mutex};
use doux::Engine;
use doux_sova::create_integration;

// 1. Create the doux engine
let engine = Arc::new(Mutex::new(Engine::new(44100.0)));

// 2. Get initial sync time from Sova's clock
let initial_time = clock.micros();

// 3. Create integration - returns thread handle and proxy
let (handle, proxy) = create_integration(engine.clone(), initial_time);

// 4. Register proxy with Sova's device map
device_map.connect_audio_engine("doux", proxy)?;

// 5. Start your audio output loop using the engine
// (see doux/src/main.rs for cpal example)
```

## Architecture

```
Sova Scheduler
    │
    ▼
AudioEngineProxy::send(AudioEnginePayload)
    │
    ▼ (crossbeam channel)
    │
SovaReceiver thread
    │ converts HashMap → command string
    ▼
Engine::evaluate("/s/sine/freq/440/gain/0.5/...")
```

## Time Synchronization

Pass `clock.micros()` at engine startup. Sova's timetags (microseconds) are converted to engine time (seconds) relative to this initial value. Events with timetags go to Doux's scheduler for sample-accurate playback.

## DouxConfig Methods

| Method | Description |
|--------|-------------|
| `with_output_device(name)` | Set output device by name |
| `with_input_device(name)` | Set input device by name |
| `with_channels(n)` | Set number of output channels |
| `with_buffer_size(n)` | Set audio buffer size in frames |
| `with_sample_path(path)` | Add a single sample directory |
| `with_sample_paths(paths)` | Add multiple sample directories |

## DouxManager Methods

| Method | Description |
|--------|-------------|
| `start(time)` | Start audio streams, returns proxy |
| `stop()` | Stop audio streams |
| `restart(config, time)` | Restart with new configuration |
| `hush()` | Release all voices gracefully |
| `panic()` | Immediately stop all voices |
| `add_sample_path(path)` | Add sample directory |
| `rescan_samples()` | Rescan all sample directories |
| `clear_samples()` | Clear sample pool |
| `state()` | Returns `AudioEngineState` |
| `is_running()` | Check if streams are active |
| `sample_rate()` | Get current sample rate |
| `channels()` | Get number of channels |
| `config()` | Get current configuration |
| `engine_handle()` | Access engine for telemetry |
| `scope_capture()` | Get oscilloscope buffer |

## Re-exported Types

### AudioEngineState

```rust
pub struct AudioEngineState {
    pub active_voices: u32,
    pub cpu_percent: f32,
    pub output_level: f32,
}
```

### DouxError

Error type for manager operations (device not found, stream errors, etc).

### ScopeCapture

Buffer containing oscilloscope data from `scope_capture()`.

## Parameters

All Sova Dirt parameters map directly to Doux event fields via `Event::parse()`. No manual mapping required - add new parameters to Doux and they work automatically.
