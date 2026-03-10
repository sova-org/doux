use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use crossbeam_channel::Sender;
use ringbuf::{traits::*, HeapRb};
use serde::{Deserialize, Serialize};

use doux::audio::{
    default_input_device, default_output_device, find_input_device, find_output_device,
    max_output_channels,
};
use doux::config::DouxConfig;
use doux::error::DouxError;
use doux::telemetry::EngineMetrics;
use doux::Engine;
use sova_core::clock::SyncTime;
use sova_core::protocol::audio_engine_proxy::AudioEngineProxy;

use crate::receiver::SovaReceiver;
use crate::scope::ScopeCapture;
use crate::time::TimeConverter;

pub enum AudioCmd {
    Evaluate(String),
    Hush,
    Panic,
    LoadSamples(Vec<doux::sampling::SampleEntry>),
    ClearSamples,
    RescanSamples(Vec<PathBuf>),
    #[cfg(feature = "soundfont")]
    LoadSoundfont(PathBuf),
}

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
    pending_engine: Option<Engine>,
    metrics: Arc<EngineMetrics>,
    cmd_tx: Option<Sender<AudioCmd>>,
    config: DouxConfig,
    sample_rate: f32,
    actual_channels: usize,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
    receiver_handle: Option<JoinHandle<()>>,
    scope: Option<Arc<ScopeCapture>>,
    device_lost: Arc<AtomicBool>,
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

        let metrics = Arc::new(EngineMetrics::default());
        let mut engine =
            Engine::new_with_metrics(sample_rate, actual_channels, config.max_voices, Arc::clone(&metrics));

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        Ok(Self {
            pending_engine: Some(engine),
            metrics,
            cmd_tx: None,
            config,
            sample_rate,
            actual_channels,
            output_stream: None,
            input_stream: None,
            receiver_handle: None,
            scope: None,
            device_lost: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(
        &mut self,
        initial_sync_time: SyncTime,
    ) -> Result<AudioEngineProxy, DouxError> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let proxy = AudioEngineProxy::new(tx);

        self.build_streams()?;

        let cmd_tx = self.cmd_tx.as_ref().expect("build_streams sets cmd_tx");
        let time_converter = TimeConverter::new(initial_sync_time);
        let receiver = SovaReceiver::new(cmd_tx.clone(), rx, time_converter, self.sample_rate as f64);
        let handle = std::thread::spawn(move || receiver.run());
        self.receiver_handle = Some(handle);

        Ok(proxy)
    }

    fn build_streams(&mut self) -> Result<(), DouxError> {
        let mut engine = self
            .pending_engine
            .take()
            .expect("pending_engine must be set before build_streams");

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

        const INPUT_BUFFER_SIZE: usize = 8192;
        let (input_producer, input_consumer) = HeapRb::<f32>::new(INPUT_BUFFER_SIZE).split();

        let input_device = match &self.config.input_device {
            Some(spec) => find_input_device(spec),
            None => default_input_device(),
        };

        let input_channels: usize = input_device
            .as_ref()
            .and_then(|dev| dev.default_input_config().ok())
            .map_or(0, |cfg| cfg.channels() as usize);

        let flag = Arc::clone(&self.device_lost);
        self.input_stream = input_device.and_then(|input_dev| {
            let input_cfg = match input_dev.default_input_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("input config error: {e}");
                    return None;
                }
            };
            if input_cfg.sample_rate() != device_config.sample_rate() {
                eprintln!(
                    "warning: input sample rate ({}Hz) differs from output ({}Hz)",
                    input_cfg.sample_rate(),
                    device_config.sample_rate()
                );
            }
            let mut input_producer = input_producer;
            let flag = Arc::clone(&flag);
            let stream = input_dev
                .build_input_stream(
                    &input_cfg.into(),
                    move |data: &[f32], _| {
                        input_producer.push_slice(data);
                    },
                    move |err| {
                        eprintln!("input stream error: {err}");
                        flag.store(true, Ordering::Release);
                    },
                    None,
                )
                .ok()?;
            stream.play().ok()?;
            Some(stream)
        });

        let scope = Arc::new(ScopeCapture::new());
        let scope_clone = Arc::clone(&scope);

        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<AudioCmd>();

        // Update sample rate on engine if it changed during reconnect
        engine.sr = self.sample_rate;
        engine.isr = 1.0 / self.sample_rate;

        let mut input_consumer = input_consumer;
        let mut live_scratch = vec![0.0f32; 4096];
        let sample_rate = self.sample_rate;
        let output_channels = self.actual_channels;
        let flag = Arc::clone(&self.device_lost);

        let output_stream = output_device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    // Drain command channel (lock-free)
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            AudioCmd::Evaluate(s) => { engine.evaluate(&s); }
                            AudioCmd::Hush => engine.hush(),
                            AudioCmd::Panic => engine.panic(),
                            AudioCmd::LoadSamples(samples) => {
                                engine.sample_index.extend(samples);
                            }
                            AudioCmd::ClearSamples => {
                                engine.sample_index.clear();
                            }
                            AudioCmd::RescanSamples(paths) => {
                                engine.sample_index.clear();
                                for path in &paths {
                                    let index = doux::sampling::scan_samples_dir(path);
                                    engine.sample_index.extend(index);
                                }
                            }
                            #[cfg(feature = "soundfont")]
                            AudioCmd::LoadSoundfont(path) => {
                                if let Err(e) = engine.load_soundfont(&path) {
                                    eprintln!("Failed to load soundfont {}: {e}", path.display());
                                }
                            }
                        }
                    }

                    // Fill live input buffer
                    let buffer_samples = data.len() / output_channels;
                    let stereo_len = buffer_samples * 2;
                    if live_scratch.len() < stereo_len {
                        live_scratch.resize(stereo_len, 0.0);
                    }
                    match input_channels {
                        0 => {
                            live_scratch[..stereo_len].fill(0.0);
                        }
                        1 => {
                            for i in 0..buffer_samples {
                                let s = input_consumer.try_pop().unwrap_or(0.0);
                                live_scratch[i * 2] = s;
                                live_scratch[i * 2 + 1] = s;
                            }
                        }
                        2 => {
                            for sample in &mut live_scratch[..stereo_len] {
                                *sample = input_consumer.try_pop().unwrap_or(0.0);
                            }
                        }
                        _ => {
                            for i in 0..buffer_samples {
                                let l = input_consumer.try_pop().unwrap_or(0.0);
                                let r = input_consumer.try_pop().unwrap_or(0.0);
                                for _ in 2..input_channels {
                                    input_consumer.try_pop();
                                }
                                live_scratch[i * 2] = l;
                                live_scratch[i * 2 + 1] = r;
                            }
                        }
                    }
                    let excess = input_consumer.occupied_len().saturating_sub(INPUT_BUFFER_SIZE / 2);
                    for _ in 0..excess {
                        input_consumer.try_pop();
                    }

                    let buffer_time_ns = (buffer_samples as f64 / sample_rate as f64 * 1e9) as u64;
                    engine.metrics.load.set_buffer_time(buffer_time_ns);
                    engine.process_block(data, &[], &live_scratch[..stereo_len]);

                    for chunk in data.chunks(output_channels) {
                        if output_channels >= 2 {
                            scope_clone.push_stereo(chunk[0], chunk[1]);
                        } else {
                            scope_clone.push_mono(chunk[0]);
                        }
                    }
                },
                move |err| {
                    eprintln!("output stream error: {err}");
                    flag.store(true, Ordering::Release);
                },
                None,
            )
            .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        output_stream
            .play()
            .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        self.output_stream = Some(output_stream);
        self.scope = Some(scope);
        self.cmd_tx = Some(cmd_tx);

        Ok(())
    }

    pub fn needs_reconnect(&self) -> bool {
        self.device_lost.load(Ordering::Acquire)
    }

    pub fn device_lost_flag(&self) -> &Arc<AtomicBool> {
        &self.device_lost
    }

    pub fn reconnect_streams(&mut self) -> Result<(), DouxError> {
        self.device_lost.store(false, Ordering::Release);
        // Drop old streams — this drops the audio callback and the engine with it
        self.output_stream = None;
        self.input_stream = None;
        self.scope = None;
        self.cmd_tx = None;

        let output_device = resolve_output_device(&self.config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, self.config.channels);

        self.sample_rate = sample_rate;
        self.actual_channels = actual_channels;

        // Create fresh engine for the new audio callback
        self.metrics = Arc::new(EngineMetrics::default());
        let mut engine = Engine::new_with_metrics(
            sample_rate,
            actual_channels,
            self.config.max_voices,
            Arc::clone(&self.metrics),
        );
        for path in &self.config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }
        self.pending_engine = Some(engine);

        self.build_streams()
    }

    pub fn stop(&mut self) {
        self.output_stream = None;
        self.input_stream = None;
        self.scope = None;
        self.cmd_tx = None;

        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn restart(
        &mut self,
        config: DouxConfig,
        initial_sync_time: SyncTime,
    ) -> Result<AudioEngineProxy, DouxError> {
        self.stop();
        self.device_lost.store(false, Ordering::Release);

        let output_device = resolve_output_device(&config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        let metrics = Arc::new(EngineMetrics::default());
        let mut engine = Engine::new_with_metrics(
            sample_rate,
            actual_channels,
            config.max_voices,
            Arc::clone(&metrics),
        );

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        self.pending_engine = Some(engine);
        self.metrics = metrics;
        self.config = config;
        self.sample_rate = sample_rate;
        self.actual_channels = actual_channels;

        self.start(initial_sync_time)
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
        self.output_stream.is_some() && !self.needs_reconnect()
    }

    pub fn state(&self) -> AudioEngineState {
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
            active_voices: self.metrics.active_voices.load(Ordering::Relaxed) as usize,
            sample_paths: self.config.sample_paths.clone(),
            error: if self.needs_reconnect() {
                Some("Audio device disconnected".to_string())
            } else {
                None
            },
            cpu_load: self.metrics.load.get_load(),
            peak_voices: self.metrics.peak_voices.load(Ordering::Relaxed) as usize,
            max_voices: self.config.max_voices,
            schedule_depth: self.metrics.schedule_depth.load(Ordering::Relaxed) as usize,
            sample_pool_mb: self.metrics.sample_pool_mb(),
        }
    }

    pub fn add_sample_path(&mut self, path: std::path::PathBuf) {
        let index = doux::sampling::scan_samples_dir(&path);
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(AudioCmd::LoadSamples(index));
        }
        self.config.sample_paths.push(path);
    }

    pub fn rescan_samples(&mut self) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(AudioCmd::RescanSamples(self.config.sample_paths.clone()));
        }
    }

    pub fn clear_samples(&mut self) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(AudioCmd::ClearSamples);
        }
    }

    pub fn hush(&self) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(AudioCmd::Hush);
        }
    }

    pub fn panic(&self) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(AudioCmd::Panic);
        }
    }

    #[cfg(feature = "soundfont")]
    pub fn load_soundfont_from_paths(&self, sample_paths: &[PathBuf]) {
        if let Some(tx) = &self.cmd_tx {
            for path in sample_paths {
                if let Some(sf2_path) = doux::soundfont::find_sf2_file(path) {
                    let _ = tx.send(AudioCmd::LoadSoundfont(sf2_path));
                    return;
                }
            }
        }
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
