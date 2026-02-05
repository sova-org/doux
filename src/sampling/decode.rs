//! Audio sample loading and directory scanning.
//!
//! Handles discovery and decoding of audio files into the engine's sample pool.
//! Supports common audio formats via Symphonia: WAV, MP3, OGG, FLAC, AAC, M4A.
//!
//! # Directory Structure
//!
//! The scanner expects samples organized as:
//!
//! ```text
//! samples/
//! ├── kick.wav           → named "kick"
//! ├── snare.wav          → named "snare"
//! └── hats/              → folder creates numbered entries
//!     ├── closed.wav     → named "hats/0"
//!     ├── open.wav       → named "hats/1"
//!     └── pedal.wav      → named "hats/2"
//! ```
//!
//! Files within folders are sorted alphabetically and assigned sequential indices.
//!
//! # Lazy Loading
//!
//! [`scan_samples_dir`] only builds the index without decoding audio data.
//! Actual decoding happens on first use via [`load_sample_file`], keeping
//! startup fast even with large sample libraries.

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::registry::SampleData;
use super::sample::SampleEntry;

/// Default base frequency assigned to loaded samples (C2 = 65.406 Hz).
///
/// Samples are assumed to be pitched at this frequency unless overridden.
/// Used for pitch-shifting calculations during playback.
const DEFAULT_BASE_FREQ: f32 = 65.406;

/// Supported audio file extensions.
const AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "ogg", "flac", "aac", "m4a"];

/// Checks if a file path has a supported audio extension.
fn is_audio_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        AUDIO_EXTENSIONS
            .iter()
            .any(|ext| e.eq_ignore_ascii_case(ext))
    })
}

/// Scans a directory for audio samples without loading audio data.
///
/// Builds an index of [`SampleEntry`] with paths and names. Audio data
/// remains unloaded (`loaded: None`) until explicitly requested.
///
/// Top-level audio files are named by their stem (filename without extension).
/// Subdirectories create grouped entries named `folder/index` where index
/// is the alphabetical position within that folder.
///
/// Prints a summary of discovered samples and folders to stdout.
pub fn scan_samples_dir(dir: &Path) -> Vec<SampleEntry> {
    let mut entries = Vec::new();
    let mut _folder_count = 0;

    let items = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read directory {}: {e}", dir.display());
            return entries;
        }
    };

    let mut paths: Vec<_> = items.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();

    for item in paths {
        if item.is_dir() {
            let folder_name = item
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let sub_entries = match std::fs::read_dir(&item) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let mut files: Vec<_> = sub_entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| is_audio_file(p))
                .collect();

            files.sort();

            if !files.is_empty() {
                _folder_count += 1;
            }

            for (i, path) in files.into_iter().enumerate() {
                let name = format!("{folder_name}/{i}");
                entries.push(SampleEntry { path, name });
            }
        } else if is_audio_file(&item) {
            let name = item
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            entries.push(SampleEntry { path: item, name });
        }
    }

    entries
}

/// Decodes an audio file into SampleData without loading into Engine.
///
/// Handles format detection, decoding, and sample rate conversion automatically.
/// Returns immutable SampleData suitable for the lock-free registry.
///
/// # Errors
///
/// Returns `Err` if:
/// - File cannot be opened or read
/// - Format is unsupported or corrupted
/// - No audio track is found
/// - Decoding fails completely (partial decode errors are skipped)
pub fn decode_sample_file(path: &Path, target_sr: f32) -> Result<SampleData, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let codec_params = &track.codec_params;
    let channels = codec_params.channels.map(|c| c.count()).unwrap_or(1) as u8;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100) as f32;

    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("Failed to read packet: {e}")),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("Decode error: {e}")),
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;

        let buf = sample_buf.get_or_insert_with(|| SampleBuffer::<f32>::new(duration, spec));
        buf.copy_interleaved_ref(decoded);

        samples.extend_from_slice(buf.samples());
    }

    if samples.is_empty() {
        return Err("No samples decoded".to_string());
    }

    let resampled = if (sample_rate - target_sr).abs() > 1.0 {
        resample_linear(&samples, channels as usize, sample_rate, target_sr)
    } else {
        samples
    };

    Ok(SampleData::new(resampled, channels, DEFAULT_BASE_FREQ))
}

/// Maximum frames to decode for head preloading (~93ms at 44.1kHz).
pub const HEAD_FRAMES: usize = 4096;

/// Decodes only the first [`HEAD_FRAMES`] of an audio file.
///
/// If the file is shorter than HEAD_FRAMES, the entire file is decoded.
/// Used for head-preloading: the attack portion lives in RAM so playback
/// can start instantly while the rest streams from disk on demand.
pub fn decode_sample_head(path: &Path, target_sr: f32) -> Result<SampleData, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe format: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let codec_params = &track.codec_params;
    let channels = codec_params.channels.map(|c| c.count()).unwrap_or(1) as u8;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100) as f32;
    let max_interleaved = HEAD_FRAMES * channels as usize;

    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {e}"))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        if samples.len() >= max_interleaved {
            samples.truncate(max_interleaved);
            break;
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("Failed to read packet: {e}")),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("Decode error: {e}")),
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;

        let buf = sample_buf.get_or_insert_with(|| SampleBuffer::<f32>::new(duration, spec));
        buf.copy_interleaved_ref(decoded);

        samples.extend_from_slice(buf.samples());
    }

    if samples.is_empty() {
        return Err("No samples decoded".to_string());
    }

    // Truncate to exact head limit after final packet
    if samples.len() > max_interleaved {
        samples.truncate(max_interleaved);
    }

    let resampled = if (sample_rate - target_sr).abs() > 1.0 {
        resample_linear(&samples, channels as usize, sample_rate, target_sr)
    } else {
        samples
    };

    Ok(SampleData::new(resampled, channels, DEFAULT_BASE_FREQ))
}

/// Resamples interleaved audio using linear interpolation.
///
/// Simple but fast resampling suitable for non-critical applications.
/// For higher quality, consider using a dedicated resampling library like rubato.
fn resample_linear(samples: &[f32], channels: usize, from_sr: f32, to_sr: f32) -> Vec<f32> {
    let ratio = to_sr / from_sr;
    let in_frames = samples.len() / channels;
    let out_frames = (in_frames as f32 * ratio) as usize;
    let mut output = vec![0.0; out_frames * channels];

    for out_frame in 0..out_frames {
        let in_pos = out_frame as f32 / ratio;
        let in_frame = in_pos as usize;
        let next_frame = (in_frame + 1).min(in_frames - 1);
        let frac = in_pos - in_frame as f32;

        for ch in 0..channels {
            let s0 = samples[in_frame * channels + ch];
            let s1 = samples[next_frame * channels + ch];
            output[out_frame * channels + ch] = s0 + frac * (s1 - s0);
        }
    }

    output
}
