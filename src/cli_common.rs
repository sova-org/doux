use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Receiver;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::audio::{find_device, host_controls_buffer_size, list_hosts, max_output_channels};
use crate::{AudioCmd, Engine};

const INPUT_BUFFER_SIZE: usize = 8192;

pub struct OutputConfig {
    pub stream_config: cpal::StreamConfig,
    pub output_channels: usize,
    pub sample_rate: f32,
}

pub fn resolve_output_config(
    host: &cpal::Host,
    output_spec: Option<&str>,
    requested_channels: u16,
    buffer_size: Option<u32>,
) -> OutputConfig {
    let device = match output_spec {
        Some(spec) => host
            .output_devices()
            .ok()
            .and_then(|d| find_device(d, spec))
            .unwrap_or_else(|| panic!("output device '{spec}' not found")),
        None => host.default_output_device().expect("no output device"),
    };

    let max_ch = max_output_channels(&device);
    let output_channels = (requested_channels as usize).min(max_ch as usize);
    if requested_channels as usize > output_channels {
        eprintln!(
            "Warning: device supports max {} channels, using that instead of {}",
            max_ch, requested_channels
        );
    }

    let default_config = device.default_output_config().expect("no output config");
    let sample_rate = default_config.sample_rate() as f32;

    let buf_size = match buffer_size {
        Some(buf) if !host_controls_buffer_size(host) => cpal::BufferSize::Fixed(buf),
        Some(_) => {
            eprintln!("Note: host controls buffer size, ignoring -b flag");
            cpal::BufferSize::Default
        }
        None => cpal::BufferSize::Default,
    };

    OutputConfig {
        stream_config: cpal::StreamConfig {
            channels: output_channels as u16,
            sample_rate: default_config.sample_rate(),
            buffer_size: buf_size,
        },
        output_channels,
        sample_rate,
    }
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
    mut engine: Engine,
    cmd_rx: Receiver<AudioCmd>,
) -> AudioStreams {
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

    let input_buffer_size = INPUT_BUFFER_SIZE * (input_channels.max(2) / 2);
    let (mut input_producer, mut input_consumer) = HeapRb::<f32>::new(input_buffer_size).split();

    engine.input_channels = input_channels;

    let flag = Arc::clone(params.device_lost);
    let input_stream = input_device.and_then(|input_dev| {
        let input_config = input_dev.default_input_config().ok()?;
        let flag = Arc::clone(&flag);
        let stream = input_dev
            .build_input_stream(
                &input_config.into(),
                move |data: &[f32], _| {
                    input_producer.push_slice(data);
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
            .unwrap_or_else(|| panic!("output device '{spec}' not found")),
        None => params
            .host
            .default_output_device()
            .expect("no output device"),
    };

    let flag = Arc::clone(params.device_lost);
    let nch_in = input_channels.max(1);
    let sr = params.config.sample_rate;
    let ch = params.config.output_channels;
    let mut scratch = vec![0.0f32; 1024];

    let output_stream = device
        .build_output_stream(
            &params.config.stream_config,
            move |data: &mut [f32], _| {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        AudioCmd::Evaluate(s) => { engine.evaluate(&s); }
                        AudioCmd::Hush => engine.hush(),
                        AudioCmd::Panic => engine.panic(),
                    }
                }

                let buffer_samples = data.len() / ch;
                let raw_len = buffer_samples * nch_in;
                if scratch.len() < raw_len {
                    scratch.resize(raw_len, 0.0);
                }
                scratch[..raw_len].fill(0.0);
                input_consumer.pop_slice(&mut scratch[..raw_len]);

                let buffer_time_ns = (buffer_samples as f64 / sr as f64 * 1e9) as u64;
                engine.metrics.load.set_buffer_time(buffer_time_ns);
                engine.process_block(data, &[], &scratch[..raw_len]);
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
        .unwrap();

    output_stream.play().unwrap();

    println!(
        "Output: {} @ {}Hz, {}ch",
        device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_default(),
        sr as u32,
        ch,
    );

    AudioStreams {
        output: output_stream,
        input: input_stream,
    }
}

pub fn recreate_engine(
    sample_rate: f32,
    output_channels: usize,
    max_voices: usize,
    block_size: usize,
    sample_index: &[crate::sampling::SampleEntry],
    sample_registry: &Arc<crate::sampling::SampleRegistry>,
    #[cfg(feature = "soundfont")] gm_bank: &Option<crate::soundfont::GmBank>,
) -> Engine {
    let mut engine = Engine::new_with_channels(sample_rate, output_channels, max_voices, block_size);
    engine.sample_index = sample_index.to_vec();
    engine.sample_registry = Arc::clone(sample_registry);
    #[cfg(feature = "soundfont")]
    {
        engine.gm_bank = gm_bank.clone();
    }
    engine
}
