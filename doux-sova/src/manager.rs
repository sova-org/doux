use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use crossbeam_channel::Receiver;
use serde::{Deserialize, Serialize};

use doux::audio::{
    default_input_device, default_output_device, find_input_device, find_output_device,
    max_output_channels,
};
use doux::config::DouxConfig;
use doux::error::DouxError;
use doux::Engine;

use crate::receiver::SovaReceiver;
use crate::scope::ScopeCapture;
use crate::time::TimeConverter;
use crate::types::{AudioPayload, SyncTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEngineState {
    pub running: bool,
    pub device: Option<String>,
    pub sample_rate: f32,
    pub channels: usize,
    pub buffer_size: Option<u32>,
    pub active_voices: usize,
    pub sample_paths: Vec<PathBuf>,
    pub error: Option<String>,
    pub cpu_load: f32,
    pub peak_voices: usize,
    pub max_voices: usize,
    pub schedule_depth: usize,
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

pub struct DouxManager {
    engine: Arc<Mutex<Engine>>,
    config: DouxConfig,
    sample_rate: f32,
    actual_channels: usize,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
    receiver_handle: Option<JoinHandle<()>>,
    scope: Option<Arc<ScopeCapture>>,
}

fn resolve_output_device(config: &DouxConfig) -> Result<Device, DouxError> {
    match &config.output_device {
        Some(spec) => {
            find_output_device(spec).ok_or_else(|| DouxError::DeviceNotFound(spec.clone()))
        }
        None => default_output_device().ok_or(DouxError::NoDefaultDevice),
    }
}

fn get_device_config(device: &Device) -> Result<(SupportedStreamConfig, f32), DouxError> {
    let config = device
        .default_output_config()
        .map_err(|e| DouxError::DeviceConfigError(e.to_string()))?;
    let sample_rate = config.sample_rate() as f32;
    Ok((config, sample_rate))
}

fn compute_channels(device: &Device, requested: u16) -> usize {
    let max_ch = max_output_channels(device);
    (requested as usize).min(max_ch as usize)
}

impl DouxManager {
    pub fn new(config: DouxConfig) -> Result<Self, DouxError> {
        let output_device = resolve_output_device(&config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        let mut engine = Engine::new_with_channels(sample_rate, actual_channels, config.max_voices);

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
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
            scope: None,
        })
    }

    /// Starts the audio streams and receiver thread.
    ///
    /// The receiver consumes `AudioPayload` messages from the given channel.
    pub fn start(
        &mut self,
        rx: Receiver<AudioPayload>,
        initial_sync_time: SyncTime,
    ) -> Result<(), DouxError> {
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

        let input_buffer: Arc<Mutex<VecDeque<f32>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

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

        let scope = Arc::new(ScopeCapture::new());
        let scope_clone = Arc::clone(&scope);

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
                    let buffer_samples = data.len() / output_channels;
                    let buffer_time_ns = (buffer_samples as f64 / sample_rate as f64 * 1e9) as u64;
                    engine.metrics.load.set_buffer_time(buffer_time_ns);
                    engine.process_block(data, &[], &scratch[..data.len()]);
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

        let time_converter = TimeConverter::new(initial_sync_time);
        let receiver = SovaReceiver::new(Arc::clone(&self.engine), rx, time_converter);
        let handle = std::thread::spawn(move || receiver.run());
        self.receiver_handle = Some(handle);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.output_stream = None;
        self.input_stream = None;
        self.scope = None;

        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn restart(
        &mut self,
        config: DouxConfig,
        rx: Receiver<AudioPayload>,
        initial_sync_time: SyncTime,
    ) -> Result<(), DouxError> {
        self.stop();

        let output_device = resolve_output_device(&config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        let mut engine = Engine::new_with_channels(sample_rate, actual_channels, config.max_voices);

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        self.engine = Arc::new(Mutex::new(engine));
        self.config = config;
        self.sample_rate = sample_rate;
        self.actual_channels = actual_channels;

        self.start(rx, initial_sync_time)
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.actual_channels
    }

    pub fn config(&self) -> &DouxConfig {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        self.output_stream.is_some()
    }

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

    pub fn add_sample_path(&mut self, path: std::path::PathBuf) {
        let index = doux::sampling::scan_samples_dir(&path);
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.extend(index);
        }
        self.config.sample_paths.push(path);
    }

    pub fn rescan_samples(&mut self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.clear();
            for path in &self.config.sample_paths {
                let index = doux::sampling::scan_samples_dir(path);
                engine.sample_index.extend(index);
            }
        }
    }

    pub fn clear_samples(&mut self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.sample_index.clear();
        }
    }

    pub fn hush(&self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.hush();
        }
    }

    pub fn panic(&self) {
        if let Ok(mut engine) = self.engine.lock() {
            engine.panic();
        }
    }

    pub fn engine_handle(&self) -> Arc<Mutex<Engine>> {
        Arc::clone(&self.engine)
    }

    pub fn scope_capture(&self) -> Option<Arc<ScopeCapture>> {
        self.scope.clone()
    }
}

impl Drop for DouxManager {
    fn drop(&mut self) {
        self.stop();
    }
}
