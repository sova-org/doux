//! DouxManager - Lifecycle management for the Doux audio engine with Sova integration.
//!
//! Provides a high-level API for managing the complete audio engine lifecycle,
//! including device selection, stream creation, and Sova scheduler integration.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};

use doux::audio::{
    default_input_device, default_output_device, find_input_device, find_output_device,
    max_output_channels,
};
use doux::config::DouxConfig;
use doux::error::DouxError;
use doux::Engine;

use sova_core::clock::SyncTime;
use sova_core::protocol::audio_engine_proxy::{AudioEnginePayload, AudioEngineProxy};

use crate::receiver::SovaReceiver;
use crate::scope::ScopeCapture;
use crate::time::TimeConverter;

/// Snapshot of the audio engine state for external visibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEngineState {
    /// Whether audio streams are currently running.
    pub running: bool,
    /// Name of the output device, or None if using system default.
    pub device: Option<String>,
    /// Sample rate in Hz.
    pub sample_rate: f32,
    /// Number of output channels.
    pub channels: usize,
    /// Requested buffer size in samples, if explicitly set.
    pub buffer_size: Option<u32>,
    /// Number of currently playing voices.
    pub active_voices: usize,
    /// Configured sample directory paths.
    pub sample_paths: Vec<PathBuf>,
    /// Last error message, if any.
    pub error: Option<String>,
    /// CPU load as a fraction (0.0 to 1.0+).
    pub cpu_load: f32,
    /// Peak number of voices seen since reset.
    pub peak_voices: usize,
    /// Maximum allowed voices.
    pub max_voices: usize,
    /// Number of events in the schedule queue.
    pub schedule_depth: usize,
    /// Total memory used by sample pool in megabytes.
    pub sample_pool_mb: f32,
}

impl Default for AudioEngineState {
    fn default() -> Self {
        Self {
            running: false,
            device: None,
            sample_rate: 0.0,
            channels: 0,
            buffer_size: None,
            active_voices: 0,
            sample_paths: Vec::new(),
            error: None,
            cpu_load: 0.0,
            peak_voices: 0,
            max_voices: doux::types::DEFAULT_MAX_VOICES,
            schedule_depth: 0,
            sample_pool_mb: 0.0,
        }
    }
}

/// Manages the Doux audio engine lifecycle with Sova integration.
///
/// Handles creating, starting, stopping, and restarting the audio engine
/// with different configurations.
pub struct DouxManager {
    /// The audio synthesis engine, shared with the audio callback thread.
    engine: Arc<Mutex<Engine>>,
    /// Current configuration (device, channels, sample paths).
    config: DouxConfig,
    /// Actual sample rate from the audio device.
    sample_rate: f32,
    /// Actual channel count (may be clamped to device maximum).
    actual_channels: usize,
    /// Handle to the CPAL output stream, None when stopped.
    output_stream: Option<Stream>,
    /// Handle to the CPAL input stream, None when stopped or no input.
    input_stream: Option<Stream>,
    /// Handle to the Sova receiver thread.
    receiver_handle: Option<JoinHandle<()>>,
    /// Sender end of the channel to the receiver, dropped to signal shutdown.
    proxy_sender: Option<Sender<AudioEnginePayload>>,
    /// Scope capture for oscilloscope display.
    scope: Option<Arc<ScopeCapture>>,
}

/// Resolves the output device from config, returning an error if not found.
fn resolve_output_device(config: &DouxConfig) -> Result<Device, DouxError> {
    match &config.output_device {
        Some(spec) => {
            find_output_device(spec).ok_or_else(|| DouxError::DeviceNotFound(spec.clone()))
        }
        None => default_output_device().ok_or(DouxError::NoDefaultDevice),
    }
}

/// Gets the device configuration and extracts the sample rate.
fn get_device_config(device: &Device) -> Result<(SupportedStreamConfig, f32), DouxError> {
    let config = device
        .default_output_config()
        .map_err(|e| DouxError::DeviceConfigError(e.to_string()))?;
    let sample_rate = config.sample_rate().0 as f32;
    Ok((config, sample_rate))
}

/// Computes the actual channel count, clamped to the device maximum.
fn compute_channels(device: &Device, requested: u16) -> usize {
    let max_ch = max_output_channels(device);
    (requested as usize).min(max_ch as usize)
}

impl DouxManager {
    /// Creates a new DouxManager with the given configuration.
    ///
    /// This resolves the audio device and creates the engine, but does not
    /// start the audio streams. Call `start()` to begin audio processing.
    pub fn new(config: DouxConfig) -> Result<Self, DouxError> {
        let output_device = resolve_output_device(&config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        // Create engine
        let mut engine = Engine::new_with_channels(sample_rate, actual_channels, config.max_voices);

        // Load sample directories
        for path in &config.sample_paths {
            let index = doux::loader::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            config,
            sample_rate,
            actual_channels,
            output_stream: None,
            input_stream: None,
            receiver_handle: None,
            proxy_sender: None,
            scope: None,
        })
    }

    /// Starts the audio streams and returns an AudioEngineProxy for Sova.
    ///
    /// The proxy can be registered with Sova's device map to receive events.
    pub fn start(&mut self, initial_sync_time: SyncTime) -> Result<AudioEngineProxy, DouxError> {
        let output_device = resolve_output_device(&self.config)?;
        let (device_config, _) = get_device_config(&output_device)?;

        let stream_config = cpal::StreamConfig {
            channels: self.actual_channels as u16,
            sample_rate: device_config.sample_rate(),
            buffer_size: self
                .config
                .buffer_size
                .map(cpal::BufferSize::Fixed)
                .unwrap_or(cpal::BufferSize::Default),
        };

        // Ring buffer for live audio input
        let input_buffer: Arc<Mutex<VecDeque<f32>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

        // Set up input stream if configured
        let input_device = match &self.config.input_device {
            Some(spec) => find_input_device(spec),
            None => default_input_device(),
        };

        self.input_stream = input_device.and_then(|input_dev| {
            let input_config = input_dev.default_input_config().ok()?;
            let buf = Arc::clone(&input_buffer);
            let stream = input_dev
                .build_input_stream(
                    &input_config.into(),
                    move |data: &[f32], _| {
                        let mut b = buf.lock().unwrap();
                        for &sample in data {
                            b.push_back(sample);
                            if b.len() > 8192 {
                                b.pop_front();
                            }
                        }
                    },
                    |err| eprintln!("input stream error: {err}"),
                    None,
                )
                .ok()?;
            stream.play().ok()?;
            Some(stream)
        });

        // Create scope capture for oscilloscope
        let scope = Arc::new(ScopeCapture::new());
        let scope_clone = Arc::clone(&scope);

        // Build output stream
        let engine_clone = Arc::clone(&self.engine);
        let input_buf_clone = Arc::clone(&input_buffer);
        let live_scratch: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0; 1024]));
        let live_scratch_clone = Arc::clone(&live_scratch);
        let sample_rate = self.sample_rate;
        let output_channels = self.actual_channels;

        let output_stream = output_device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    let mut buf = input_buf_clone.lock().unwrap();
                    let mut scratch = live_scratch_clone.lock().unwrap();
                    if scratch.len() < data.len() {
                        scratch.resize(data.len(), 0.0);
                    }
                    for sample in scratch[..data.len()].iter_mut() {
                        *sample = buf.pop_front().unwrap_or(0.0);
                    }
                    drop(buf);
                    let mut engine = engine_clone.lock().unwrap();
                    // Set buffer time budget for CPU load measurement
                    let buffer_samples = data.len() / output_channels;
                    let buffer_time_ns = (buffer_samples as f64 / sample_rate as f64 * 1e9) as u64;
                    engine.metrics.load.set_buffer_time(buffer_time_ns);
                    engine.process_block(data, &[], &scratch[..data.len()]);
                    // Capture samples for oscilloscope (zero-allocation path)
                    for chunk in data.chunks(output_channels) {
                        if output_channels >= 2 {
                            scope_clone.push_stereo(chunk[0], chunk[1]);
                        } else {
                            scope_clone.push_mono(chunk[0]);
                        }
                    }
                },
                |err| eprintln!("output stream error: {err}"),
                None,
            )
            .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        output_stream
            .play()
            .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        self.output_stream = Some(output_stream);
        self.scope = Some(scope);

        // Create Sova integration
        let (tx, rx) = crossbeam_channel::unbounded();
        let time_converter = TimeConverter::new(initial_sync_time);
        let receiver = SovaReceiver::new(Arc::clone(&self.engine), rx, time_converter);
        let handle = std::thread::spawn(move || receiver.run());

        self.receiver_handle = Some(handle);
        self.proxy_sender = Some(tx.clone());

        Ok(AudioEngineProxy::new(tx))
    }

    /// Stops all audio streams and the Sova receiver.
    pub fn stop(&mut self) {
        // Drop streams to stop audio
        self.output_stream = None;
        self.input_stream = None;
        self.scope = None;

        // Drop sender to signal receiver to stop
        self.proxy_sender = None;

        // Wait for receiver thread to finish (it will exit when channel closes)
        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.join();
        }
    }

    /// Restarts the engine with a new configuration.
    ///
    /// Stops the current engine, creates a new one with the new config,
    /// and returns a new AudioEngineProxy.
    pub fn restart(
        &mut self,
        config: DouxConfig,
        initial_sync_time: SyncTime,
    ) -> Result<AudioEngineProxy, DouxError> {
        self.stop();

        let output_device = resolve_output_device(&config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        // Create new engine
        let mut engine = Engine::new_with_channels(sample_rate, actual_channels, config.max_voices);

        for path in &config.sample_paths {
            let index = doux::loader::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        self.engine = Arc::new(Mutex::new(engine));
        self.config = config;
        self.sample_rate = sample_rate;
        self.actual_channels = actual_channels;

        self.start(initial_sync_time)
    }

    /// Returns the actual sample rate being used.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Returns the actual number of output channels.
    pub fn channels(&self) -> usize {
        self.actual_channels
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &DouxConfig {
        &self.config
    }

    /// Returns whether audio streams are running.
    pub fn is_running(&self) -> bool {
        self.output_stream.is_some()
    }

    /// Returns a snapshot of the current audio engine state.
    pub fn state(&self) -> AudioEngineState {
        use std::sync::atomic::Ordering;

        let (active_voices, cpu_load, peak_voices, schedule_depth, sample_pool_mb) = self
            .engine
            .lock()
            .map(|e| {
                (
                    e.active_voices,
                    e.metrics.load.get_load(),
                    e.metrics.peak_voices.load(Ordering::Relaxed) as usize,
                    e.metrics.schedule_depth.load(Ordering::Relaxed) as usize,
                    e.metrics.sample_pool_mb(),
                )
            })
            .unwrap_or((0, 0.0, 0, 0, 0.0));

        AudioEngineState {
            running: self.is_running(),
            device: self
                .config
                .output_device
                .clone()
                .or_else(|| Some("System Default".to_string())),
            sample_rate: self.sample_rate,
            channels: self.actual_channels,
            buffer_size: self.config.buffer_size,
            active_voices,
            sample_paths: self.config.sample_paths.clone(),
            error: None,
            cpu_load,
            peak_voices,
            max_voices: self.config.max_voices,
            schedule_depth,
            sample_pool_mb,
        }
    }

    /// Adds a sample directory and scans it.
    pub fn add_sample_path(&mut self, path: std::path::PathBuf) {
        let index = doux::loader::scan_samples_dir(&path);
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.extend(index);
        }
        self.config.sample_paths.push(path);
    }

    /// Rescans all configured sample directories.
    pub fn rescan_samples(&mut self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.clear();
            for path in &self.config.sample_paths {
                let index = doux::loader::scan_samples_dir(path);
                engine.sample_index.extend(index);
            }
        }
    }

    /// Clears all loaded samples.
    pub fn clear_samples(&mut self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.clear();
            engine.samples.clear();
            engine.sample_pool = doux::sample::SamplePool::new();
        }
    }

    /// Sends a hush command to release all voices.
    pub fn hush(&self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.hush();
        }
    }

    /// Sends a panic command to immediately stop all voices.
    pub fn panic(&self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.panic();
        }
    }

    /// Returns a handle to the engine for telemetry access.
    ///
    /// This allows external code to read engine metrics without holding
    /// the entire DouxManager (which is not Send due to cpal::Stream).
    pub fn engine_handle(&self) -> Arc<Mutex<Engine>> {
        Arc::clone(&self.engine)
    }

    /// Returns the scope capture for oscilloscope display.
    ///
    /// Returns None if the audio engine is not running.
    pub fn scope_capture(&self) -> Option<Arc<ScopeCapture>> {
        self.scope.clone()
    }
}

impl Drop for DouxManager {
    fn drop(&mut self) {
        self.stop();
    }
}
