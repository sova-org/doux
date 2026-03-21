use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use doux::audio::cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Host, Stream, SupportedStreamConfig};
use crossbeam_channel::Sender;
use ringbuf::{traits::*, HeapRb};
use serde::{Deserialize, Serialize};

use doux::audio::{
    find_device, get_host, host_controls_buffer_size, max_output_channels, HostSelection,
};
use doux::config::DouxConfig;
use doux::error::DouxError;
use doux::telemetry::EngineMetrics;
use doux::Engine;
use sova_core::clock::SyncTime;
use sova_core::protocol::audio_engine_proxy::AudioEngineProxy;

use crate::peaks::PeakCapture;
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
    #[serde(default = "default_volume")]
    pub volume: f32,
}

fn default_volume() -> f32 {
    1.0
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
            volume: 1.0,
        }
    }
}

pub struct DouxManager {
    pending_engine: Option<Engine>,
    metrics: Arc<EngineMetrics>,
    cmd_tx: Option<Sender<AudioCmd>>,
    config: DouxConfig,
    host_selection: HostSelection,
    sample_rate: f32,
    actual_channels: usize,
    output_stream: Option<Stream>,
    input_stream: Option<Stream>,
    receiver_handle: Option<JoinHandle<()>>,
    scope: Option<Arc<ScopeCapture>>,
    peaks: Option<Arc<PeakCapture>>,
    device_lost: Arc<AtomicBool>,
    master_gain: Arc<AtomicU32>,
}

fn parse_host_selection(host: Option<&str>) -> Result<HostSelection, DouxError> {
    match host {
        Some(s) => s
            .parse::<HostSelection>()
            .map_err(DouxError::HostNotFound),
        None => Ok(HostSelection::default()),
    }
}

fn resolve_output_device(host: &Host, config: &DouxConfig) -> Result<Device, DouxError> {
    match &config.output_device {
        Some(spec) => host
            .output_devices()
            .ok()
            .and_then(|devs| find_device(devs, spec))
            .ok_or_else(|| DouxError::DeviceNotFound(spec.clone())),
        None => host
            .default_output_device()
            .ok_or(DouxError::NoDefaultDevice),
    }
}

fn resolve_input_device(host: &Host, config: &DouxConfig) -> Option<Device> {
    match &config.input_device {
        Some(spec) => {
            let dev = host
                .input_devices()
                .ok()
                .and_then(|devs| find_device(devs, spec));
            if dev.is_none() {
                eprintln!("[doux] input device not found: {spec}");
            }
            dev
        }
        None => {
            let dev = host.default_input_device();
            match &dev {
                Some(_) => eprintln!("[doux] using default input device"),
                None => eprintln!("[doux] no input device available"),
            }
            dev
        }
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

/// Negotiate a stream config that the device actually supports.
///
/// Tries the requested (channels, sample_rate) first. If no supported config
/// range covers that combination, falls back to the device's default config.
fn negotiate_stream_config(
    device: &Device,
    requested_channels: u16,
    preferred_sample_rate: cpal::SampleRate,
    buf_size: cpal::BufferSize,
) -> Result<cpal::StreamConfig, DouxError> {
    // Check if our target is within any supported range
    let supported = device
        .supported_output_configs()
        .ok()
        .map(|configs| {
            configs.into_iter().any(|range| {
                range.channels() >= requested_channels
                    && range.min_sample_rate() <= preferred_sample_rate
                    && range.max_sample_rate() >= preferred_sample_rate
            })
        })
        .unwrap_or(false);

    if supported {
        return Ok(cpal::StreamConfig {
            channels: requested_channels,
            sample_rate: preferred_sample_rate,
            buffer_size: buf_size,
        });
    }

    // Fallback: use default config as-is
    eprintln!(
        "[doux] requested config ({requested_channels}ch @ {}Hz) not supported, falling back to device default",
        preferred_sample_rate
    );
    let default = device
        .default_output_config()
        .map_err(|e| DouxError::DeviceConfigError(e.to_string()))?;

    Ok(cpal::StreamConfig {
        channels: default.channels(),
        sample_rate: default.sample_rate(),
        buffer_size: buf_size,
    })
}

impl DouxManager {
    pub fn new(config: DouxConfig) -> Result<Self, DouxError> {
        let host_selection = parse_host_selection(config.host.as_deref())?;
        let host = get_host(host_selection.clone())?;
        let output_device = resolve_output_device(&host, &config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        let metrics = Arc::new(EngineMetrics::default());
        let block_size = config.buffer_size.map(|b| b as usize).unwrap_or(doux::types::DEFAULT_NATIVE_BLOCK_SIZE);
        let mut engine =
            Engine::new_with_metrics(sample_rate, actual_channels, config.max_voices, Arc::clone(&metrics), block_size);

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        Ok(Self {
            pending_engine: Some(engine),
            metrics,
            cmd_tx: None,
            config,
            host_selection,
            sample_rate,
            actual_channels,
            output_stream: None,
            input_stream: None,
            receiver_handle: None,
            scope: None,
            peaks: None,
            device_lost: Arc::new(AtomicBool::new(false)),
            master_gain: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        })
    }

    pub fn set_master_gain_arc(&mut self, gain: Arc<AtomicU32>) {
        self.master_gain = gain;
    }

    pub fn set_master_volume(&self, vol: f32) {
        self.master_gain
            .store(vol.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn master_volume(&self) -> f32 {
        f32::from_bits(self.master_gain.load(Ordering::Relaxed))
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

        let host = get_host(self.host_selection.clone())?;
        let output_device = resolve_output_device(&host, &self.config)?;
        let (device_config, _) = get_device_config(&output_device)?;

        let buf_size = match self.config.buffer_size {
            Some(buf) if !host_controls_buffer_size(&host) => cpal::BufferSize::Fixed(buf),
            Some(_) => {
                eprintln!("[doux] host controls buffer size, ignoring configured value");
                cpal::BufferSize::Default
            }
            None => cpal::BufferSize::Default,
        };

        let stream_config = negotiate_stream_config(
            &output_device,
            self.actual_channels as u16,
            device_config.sample_rate(),
            buf_size,
        )?;

        // Update actual values in case negotiation changed them
        self.actual_channels = stream_config.channels as usize;
        self.sample_rate = stream_config.sample_rate as f32;
        eprintln!(
            "[doux] stream config: {}ch @ {}Hz, buffer: {:?}",
            stream_config.channels, stream_config.sample_rate, stream_config.buffer_size
        );

        let input_device = resolve_input_device(&host, &self.config);

        let input_channels: usize = input_device
            .as_ref()
            .and_then(|dev| dev.default_input_config().ok())
            .map_or(0, |cfg| cfg.channels() as usize);

        eprintln!("[doux] input channels: {input_channels}");

        let input_buffer_size = 8192 * (input_channels.max(2) / 2);
        let (input_producer, input_consumer) = HeapRb::<f32>::new(input_buffer_size).split();

        let flag = Arc::clone(&self.device_lost);
        self.input_stream = input_device.and_then(|input_dev| {
            let input_cfg = match input_dev.default_input_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("[doux] input config error: {e}");
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
            let input_format = input_cfg.sample_format();
            let mut input_producer = input_producer;
            let flag = Arc::clone(&flag);

            macro_rules! build_input {
                ($T:ty) => {{
                    let mut scratch: Vec<f32> = Vec::new();
                    input_dev.build_input_stream(
                        &input_cfg.into(),
                        move |data: &[$T], _| {
                            scratch.resize(data.len(), 0.0);
                            for (dst, &src) in scratch.iter_mut().zip(data.iter()) {
                                *dst = <f32 as FromSample<$T>>::from_sample_(src);
                            }
                            input_producer.push_slice(&scratch);
                        },
                        move |err| match err {
                            cpal::StreamError::DeviceNotAvailable
                            | cpal::StreamError::StreamInvalidated => {
                                eprintln!("[doux] input device lost: {err}");
                                flag.store(true, Ordering::Release);
                            }
                            cpal::StreamError::BufferUnderrun => {
                                eprintln!("[doux] xrun");
                            }
                            other => {
                                eprintln!("[doux] input stream: {other}");
                            }
                        },
                        None,
                    )
                }};
            }

            let stream = match input_format {
                cpal::SampleFormat::F32 => build_input!(f32),
                cpal::SampleFormat::I32 => build_input!(i32),
                cpal::SampleFormat::I16 => build_input!(i16),
                format => {
                    eprintln!("[doux] unsupported input sample format: {format:?}");
                    return None;
                }
            };
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[doux] failed to build input stream: {e}");
                    return None;
                }
            };
            match stream.play() {
                Ok(()) => {
                    eprintln!("[doux] input stream started");
                    Some(stream)
                }
                Err(e) => {
                    eprintln!("[doux] failed to play input stream: {e}");
                    None
                }
            }
        });

        let scope = Arc::new(ScopeCapture::new());
        let scope_clone = Arc::clone(&scope);
        let peaks = Arc::new(PeakCapture::new(self.actual_channels));
        let peaks_clone = Arc::clone(&peaks);

        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<AudioCmd>();

        // Update sample rate and input channels on engine
        engine.sr = self.sample_rate;
        engine.isr = 1.0 / self.sample_rate;
        engine.input_channels = input_channels;

        let mut input_consumer = input_consumer;
        let mut live_scratch = vec![0.0f32; 4096];
        let sample_rate = self.sample_rate;
        let output_channels = self.actual_channels;
        let flag = Arc::clone(&self.device_lost);
        let master_gain = Arc::clone(&self.master_gain);
        let mut prev_gain = 1.0f32;
        let output_format = device_config.sample_format();

        macro_rules! build_output {
            ($T:ty) => {{
                let mut conv_buf: Vec<f32> = Vec::new();
                output_device.build_output_stream(
                    &stream_config,
                    move |data: &mut [$T], _| {
                        conv_buf.resize(data.len(), 0.0f32);

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

                        let buffer_samples = conv_buf.len() / output_channels;
                        let raw_len = buffer_samples * input_channels.max(1);
                        if live_scratch.len() < raw_len {
                            live_scratch.resize(raw_len, 0.0);
                        }
                        if input_channels == 0 {
                            live_scratch[..raw_len].fill(0.0);
                        } else {
                            for sample in &mut live_scratch[..raw_len] {
                                *sample = input_consumer.try_pop().unwrap_or(0.0);
                            }
                        }
                        let excess = input_consumer.occupied_len().saturating_sub(input_buffer_size / 2);
                        for _ in 0..excess {
                            input_consumer.try_pop();
                        }

                        let buffer_time_ns = (buffer_samples as f64 / sample_rate as f64 * 1e9) as u64;
                        engine.metrics.load.set_buffer_time(buffer_time_ns);
                        engine.process_block(&mut conv_buf, &[], &live_scratch[..raw_len]);

                        let target_gain = f32::from_bits(master_gain.load(Ordering::Relaxed));
                        if prev_gain != target_gain {
                            let num_samples = conv_buf.len();
                            let step = (target_gain - prev_gain) / num_samples as f32;
                            let mut g = prev_gain;
                            for sample in conv_buf.iter_mut() {
                                g += step;
                                *sample *= g;
                            }
                            prev_gain = target_gain;
                        } else if target_gain != 1.0 {
                            for sample in conv_buf.iter_mut() {
                                *sample *= target_gain;
                            }
                        }

                        peaks_clone.push(&conv_buf, output_channels);

                        for chunk in conv_buf.chunks(output_channels) {
                            if output_channels >= 2 {
                                scope_clone.push_stereo(chunk[0], chunk[1]);
                            } else {
                                scope_clone.push_mono(chunk[0]);
                            }
                        }

                        for (out, &src) in data.iter_mut().zip(conv_buf.iter()) {
                            *out = <$T as FromSample<f32>>::from_sample_(src);
                        }
                    },
                    move |err| match err {
                        cpal::StreamError::DeviceNotAvailable
                        | cpal::StreamError::StreamInvalidated => {
                            eprintln!("[doux] output device lost: {err}");
                            flag.store(true, Ordering::Release);
                        }
                        cpal::StreamError::BufferUnderrun => {
                            eprintln!("[doux] xrun");
                        }
                        other => {
                            eprintln!("[doux] output stream: {other}");
                        }
                    },
                    None,
                )
            }};
        }

        let output_stream = match output_format {
            cpal::SampleFormat::F32 => build_output!(f32),
            cpal::SampleFormat::I32 => build_output!(i32),
            cpal::SampleFormat::I16 => build_output!(i16),
            format => {
                return Err(DouxError::StreamCreationFailed(
                    format!("unsupported output sample format: {format:?}"),
                ));
            }
        }
        .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        output_stream
            .play()
            .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

        self.output_stream = Some(output_stream);
        self.scope = Some(scope);
        self.peaks = Some(peaks);
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
        self.peaks = None;
        self.cmd_tx = None;

        let host = get_host(self.host_selection.clone())?;
        let output_device = resolve_output_device(&host, &self.config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, self.config.channels);

        self.sample_rate = sample_rate;
        self.actual_channels = actual_channels;

        // Create fresh engine for the new audio callback
        self.metrics = Arc::new(EngineMetrics::default());
        let block_size = self.config.buffer_size.map(|b| b as usize).unwrap_or(doux::types::DEFAULT_NATIVE_BLOCK_SIZE);
        let mut engine = Engine::new_with_metrics(
            sample_rate,
            actual_channels,
            self.config.max_voices,
            Arc::clone(&self.metrics),
            block_size,
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
        self.peaks = None;
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

        let host_selection = parse_host_selection(config.host.as_deref())?;
        let host = get_host(host_selection.clone())?;
        let output_device = resolve_output_device(&host, &config)?;
        let (_, sample_rate) = get_device_config(&output_device)?;
        let actual_channels = compute_channels(&output_device, config.channels);

        let metrics = Arc::new(EngineMetrics::default());
        let block_size = config.buffer_size.map(|b| b as usize).unwrap_or(doux::types::DEFAULT_NATIVE_BLOCK_SIZE);
        let mut engine = Engine::new_with_metrics(
            sample_rate,
            actual_channels,
            config.max_voices,
            Arc::clone(&metrics),
            block_size,
        );

        for path in &config.sample_paths {
            let index = doux::sampling::scan_samples_dir(path);
            engine.sample_index.extend(index);
        }

        self.pending_engine = Some(engine);
        self.host_selection = host_selection;
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
            volume: self.master_volume(),
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

    pub fn peak_capture(&self) -> Option<Arc<PeakCapture>> {
        self.peaks.clone()
    }
}

impl Drop for DouxManager {
    fn drop(&mut self) {
        self.stop();
    }
}
