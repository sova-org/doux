use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::FromSample;
use crossbeam_channel::Receiver;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::audio::{
    find_device, get_host, host_controls_buffer_size, list_hosts, resolve_output_channels,
    print_diagnostics, HostSelection,
};
use crate::error::DouxError;
use crate::types::DEFAULT_BUFFER_SIZE;
use crate::{AudioCmd, Engine};

/// Input ring depth in host callback periods. Four covers jitter without
/// adding noticeable latency at typical buffer sizes.
const INPUT_RING_PERIODS: usize = 4;

/// CLI flags shared by all native binaries that open an audio stream.
#[derive(clap::Args)]
pub struct CommonAudioArgs {
    /// Directory containing audio samples.
    #[arg(short, long)]
    pub samples: Option<PathBuf>,

    /// List available audio devices and exit.
    #[arg(long)]
    pub list_devices: bool,

    /// Input device (name or index).
    #[arg(short, long)]
    pub input: Option<String>,

    /// Output device (name or index).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Number of output channels (default: 2, max depends on device).
    #[arg(long, default_value = "2")]
    pub channels: u16,

    /// Audio buffer size in samples (lower = less latency, higher = more stable).
    #[arg(short, long)]
    pub buffer_size: Option<u32>,

    /// Maximum polyphony (number of simultaneous voices).
    #[arg(long, default_value = "32")]
    pub max_voices: usize,

    /// DSP inner block size in samples (clamped to [1, MAX_BLOCK=256]).
    #[arg(long, default_value = "32")]
    pub dsp_block_size: usize,

    /// Audio host backend: pipewire, pulseaudio, jack, alsa, asio, or auto (default: auto).
    #[arg(long, default_value = "auto")]
    pub host: String,

    /// Run audio diagnostics and exit.
    #[arg(long)]
    pub diagnose: bool,
}

/// Outcome of host initialisation.
pub enum HostInit {
    /// User asked for `--diagnose` or `--list-devices`; the binary should exit.
    EarlyExit,
    Ready {
        host: cpal::Host,
        output_config: OutputConfig,
        buffer_size: usize,
    },
}

/// Parses host selection, runs `--diagnose` / `--list-devices` short-circuits,
/// and resolves the output configuration. Used by all interactive binaries.
pub fn init_audio_host(common: &CommonAudioArgs) -> Result<HostInit, DouxError> {
    let host_selection: HostSelection = common.host.parse().map_err(DouxError::HostNotFound)?;

    if common.diagnose {
        print_hosts();
        println!();
        print_diagnostics();
        return Ok(HostInit::EarlyExit);
    }

    let host = get_host(host_selection)?;

    if common.list_devices {
        print_devices(&host);
        return Ok(HostInit::EarlyExit);
    }

    let output_config = resolve_output_config(
        &host,
        common.output.as_deref(),
        common.channels,
        common.buffer_size,
    )?;

    let buffer_size = common
        .buffer_size
        .map(|b| b as usize)
        .unwrap_or(DEFAULT_BUFFER_SIZE);

    Ok(HostInit::Ready {
        host,
        output_config,
        buffer_size,
    })
}

/// Loads samples from `dir` into the engine.
///
/// `preload = true` decodes everything up front (blocking); otherwise sample data
/// is deferred to lazy loading. With `verbose = true`, progress is printed to stdout.
pub fn setup_engine_samples(engine: &mut Engine, dir: &Path, preload: bool, verbose: bool) {
    if verbose {
        println!("\nScanning samples from: {}", dir.display());
    }
    let index = crate::sampling::scan_samples_dir(dir);
    let count = index.len();

    if preload {
        if verbose {
            println!("Preloading {count} samples...");
        }
        let sr = engine.sample_rate();
        let registry = engine.sample_registry();
        for entry in &index {
            match crate::sampling::decode_sample_file(&entry.path, sr) {
                Ok(data) => {
                    registry.insert(entry.name.as_ref().to_string(), Arc::new(data));
                }
                Err(e) => {
                    eprintln!("Failed to preload {}: {e}", entry.name);
                }
            }
        }
        if verbose {
            println!("Preloaded {} samples\n", engine.sample_registry().len());
        }
    } else if verbose {
        println!("Found {count} samples (lazy loading enabled)\n");
    }

    engine.set_sample_index(index);

    #[cfg(feature = "soundfont")]
    if let Some(sf2_path) = crate::soundfont::find_sf2_file(dir) {
        if let Err(e) = engine.load_soundfont(&sf2_path) {
            eprintln!("Failed to load soundfont: {e}");
        }
    }
}

pub struct OutputConfig {
    pub stream_config: cpal::StreamConfig,
    pub output_channels: usize,
    pub sample_rate: f32,
    pub sample_format: cpal::SampleFormat,
}

pub fn resolve_output_config(
    host: &cpal::Host,
    output_spec: Option<&str>,
    requested_channels: u16,
    buffer_size: Option<u32>,
) -> Result<OutputConfig, DouxError> {
    let device = match output_spec {
        Some(spec) => host
            .output_devices()
            .ok()
            .and_then(|d| find_device(d, spec))
            .ok_or_else(|| DouxError::DeviceNotFound(spec.to_string()))?,
        None => host
            .default_output_device()
            .ok_or(DouxError::NoDefaultDevice)?,
    };

    let output_channels = resolve_output_channels(&device, requested_channels) as usize;

    let default_config = device
        .default_output_config()
        .map_err(|e| DouxError::DeviceConfigError(e.to_string()))?;
    let sample_rate = default_config.sample_rate() as f32;

    let buf_size = match buffer_size {
        Some(buf) if !host_controls_buffer_size(host) => cpal::BufferSize::Fixed(buf),
        Some(_) => {
            eprintln!("Note: host controls buffer size, ignoring -b flag");
            cpal::BufferSize::Default
        }
        None => cpal::BufferSize::Default,
    };

    let sample_format = default_config.sample_format();
    Ok(OutputConfig {
        stream_config: cpal::StreamConfig {
            channels: output_channels as u16,
            sample_rate: default_config.sample_rate(),
            buffer_size: buf_size,
        },
        output_channels,
        sample_rate,
        sample_format,
    })
}

pub fn print_devices(host: &cpal::Host) {
    let default_in = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));
    let default_out = host
        .default_output_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    println!("Audio host: {}", host.id().name());

    println!("\nInput devices:");
    if let Ok(devices) = host.input_devices() {
        for (i, d) in devices.enumerate() {
            let name = d
                .description()
                .map(|desc| desc.name().to_string())
                .unwrap_or_else(|_| "???".into());
            let marker = if Some(&name) == default_in.as_ref() {
                " *"
            } else {
                ""
            };
            println!("  {i}: {name}{marker}");
        }
    } else {
        println!("  (no input devices available)");
    }

    println!("\nOutput devices:");
    if let Ok(devices) = host.output_devices() {
        for (i, d) in devices.enumerate() {
            let name = d
                .description()
                .map(|desc| desc.name().to_string())
                .unwrap_or_else(|_| "???".into());
            let marker = if Some(&name) == default_out.as_ref() {
                " *"
            } else {
                ""
            };
            println!("  {i}: {name}{marker}");
        }
    } else {
        println!("  (no output devices available)");
    }
}

pub fn print_hosts() {
    println!("Available audio hosts:");
    for h in list_hosts() {
        let status = if h.available { "" } else { " (unavailable)" };
        println!("  {}{}", h.name, status);
    }
}

pub struct AudioStreams {
    pub output: cpal::Stream,
    pub input: Option<cpal::Stream>,
}

pub struct StreamParams<'a> {
    pub host: &'a cpal::Host,
    pub input_spec: Option<&'a str>,
    pub output_spec: Option<&'a str>,
    pub config: &'a OutputConfig,
    pub device_lost: &'a Arc<AtomicBool>,
}

pub fn build_audio_streams(
    params: &StreamParams,
    engine: Engine,
    cmd_rx: Receiver<AudioCmd>,
) -> Result<AudioStreams, DouxError> {
    let mut engine = engine;
    let input_device = match params.input_spec {
        Some(spec) => params
            .host
            .input_devices()
            .ok()
            .and_then(|d| find_device(d, spec)),
        None => crate::audio::default_input_device(),
    };

    let input_channels: usize = input_device
        .as_ref()
        .and_then(|dev| dev.default_input_config().ok())
        .map_or(0, |cfg| cfg.channels() as usize);

    let input_buffer_size = engine.host_buffer_size() * INPUT_RING_PERIODS * input_channels.max(2);
    let (mut input_producer, mut input_consumer) = HeapRb::<f32>::new(input_buffer_size).split();

    engine.input_channels = input_channels;

    let flag = Arc::clone(params.device_lost);
    let input_stream = input_device.and_then(|input_dev| {
        let input_config = input_dev.default_input_config().ok()?;
        let input_format = input_config.sample_format();
        let flag = Arc::clone(&flag);

        macro_rules! build_input {
            ($T:ty) => {{
                // Pre-allocate so the RT callback never grows the heap.
                // 8192 frames covers all common host buffer sizes.
                let mut scratch: Vec<f32> = vec![0.0f32; 8192];
                input_dev.build_input_stream(
                    input_config.into(),
                    move |data: &[$T], _| {
                        let usable = data.len().min(scratch.len());
                        for (dst, &src) in scratch[..usable].iter_mut().zip(data[..usable].iter()) {
                            *dst = <f32 as FromSample<$T>>::from_sample_(src);
                        }
                        input_producer.push_slice(&scratch[..usable]);
                    },
                    move |err: cpal::Error| match err.kind() {
                        cpal::ErrorKind::DeviceNotAvailable
                        | cpal::ErrorKind::StreamInvalidated => {
                            eprintln!("[doux] input device lost: {err}");
                            flag.store(true, Ordering::Release);
                        }
                        cpal::ErrorKind::Xrun => {
                            eprintln!("[doux] xrun");
                        }
                        cpal::ErrorKind::DeviceChanged => {
                            eprintln!("[doux] default input device changed; stream rerouted");
                        }
                        _ => {
                            eprintln!("[doux] input stream: {err}");
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
            _ => return None,
        }
        .ok()?;
        stream.play().ok()?;
        Some(stream)
    });

    let device = match params.output_spec {
        Some(spec) => params
            .host
            .output_devices()
            .ok()
            .and_then(|d| find_device(d, spec))
            .ok_or_else(|| DouxError::DeviceNotFound(spec.to_string()))?,
        None => params
            .host
            .default_output_device()
            .ok_or(DouxError::NoDefaultDevice)?,
    };

    let flag = Arc::clone(params.device_lost);
    let nch_in = input_channels.max(1);
    let sr = params.config.sample_rate;
    let ch = params.config.output_channels;
    // Pre-allocate so the RT callback never grows the heap. 8192 frames covers
    // all common host buffer sizes; mirrors doux-sova's manager.rs sizing.
    const MAX_BUFFER_FRAMES: usize = 8192;
    let mut scratch = vec![0.0f32; MAX_BUFFER_FRAMES * nch_in];
    let output_format = params.config.sample_format;

    macro_rules! build_output {
        ($T:ty) => {{
            let mut conv_buf: Vec<f32> = vec![0.0f32; MAX_BUFFER_FRAMES * ch];
            let mut panicked = false;
            device.build_output_stream(
                params.config.stream_config,
                move |data: &mut [$T], _| {
                    // A panic inside a cpal callback (called from C/ALSA) is UB.
                    // Wrap in catch_unwind; on panic output silence.
                    if panicked {
                        for s in data.iter_mut() {
                            *s = <$T as FromSample<f32>>::from_sample_(0.0);
                        }
                        return;
                    }
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        crate::dsp::enable_flush_to_zero();
                        // Clamp to pre-allocated size: never allocate on the RT thread.
                        let usable = data.len().min(conv_buf.len());
                        let conv = &mut conv_buf[..usable];

                        let mut cmd_budget = 64;
                        while cmd_budget > 0 {
                            match cmd_rx.try_recv() {
                                Ok(cmd) => match cmd {
                                    AudioCmd::DispatchEvent(event) => {
                                        engine.dispatch_event(event);
                                    }
                                    AudioCmd::Hush => engine.hush(),
                                    AudioCmd::Panic => engine.panic(),
                                },
                                Err(_) => break,
                            }
                            cmd_budget -= 1;
                        }

                        let buffer_samples = usable / ch;
                        let raw_len = (buffer_samples * nch_in).min(scratch.len());
                        scratch[..raw_len].fill(0.0);
                        input_consumer.pop_slice(&mut scratch[..raw_len]);

                        let buffer_time_ns = (buffer_samples as f64 / sr as f64 * 1e9) as u64;
                        engine.metrics.load.set_buffer_time(buffer_time_ns);
                        engine.process_block(conv, &[], &scratch[..raw_len]);

                        for (out, &src) in data.iter_mut().zip(conv.iter()) {
                            *out = <$T as FromSample<f32>>::from_sample_(src);
                        }
                    })); // end catch_unwind
                    if result.is_err() {
                        panicked = true;
                        eprintln!("[doux] PANIC in audio callback — outputting silence");
                        for s in data.iter_mut() {
                            *s = <$T as FromSample<f32>>::from_sample_(0.0);
                        }
                    }
                },
                move |err: cpal::Error| match err.kind() {
                    cpal::ErrorKind::DeviceNotAvailable
                    | cpal::ErrorKind::StreamInvalidated => {
                        eprintln!("[doux] output device lost: {err}");
                        flag.store(true, Ordering::Release);
                    }
                    cpal::ErrorKind::Xrun => {
                        eprintln!("[doux] xrun");
                    }
                    cpal::ErrorKind::DeviceChanged => {
                        eprintln!("[doux] default output device changed; stream rerouted");
                    }
                    _ => {
                        eprintln!("[doux] output stream: {err}");
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
            return Err(DouxError::StreamCreationFailed(format!(
                "unsupported output sample format: {format:?}"
            )));
        }
    }
    .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

    output_stream
        .play()
        .map_err(|e| DouxError::StreamCreationFailed(e.to_string()))?;

    println!(
        "Output: {} @ {}Hz, {}ch",
        device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_default(),
        sr as u32,
        ch,
    );

    Ok(AudioStreams {
        output: output_stream,
        input: input_stream,
    })
}
