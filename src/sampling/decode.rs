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
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

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

            for (i, path) in files.into_iter().enumerate() {
                let name = format!("{folder_name}/{i}");
                entries.push(SampleEntry {
                    path: Arc::new(path),
                    name: Arc::from(name),
                });
            }
        } else if is_audio_file(&item) {
            let name = item
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            entries.push(SampleEntry {
                path: Arc::new(item),
                name: Arc::from(name),
            });
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
    let channels = codec_params.channels.map(|c| c.count().max(1)).unwrap_or(1) as u8;
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

    // This decoder reads the whole file, so its own length is the source length.
    let source_frames = (samples.len() / channels as usize) as u32;
    let (resampled, resample_ratio) = if (sample_rate - target_sr).abs() > 1.0 {
        (
            resample_linear(&samples, channels as usize, sample_rate, target_sr),
            target_sr / sample_rate,
        )
    } else {
        (samples, 1.0)
    };

    Ok(
        SampleData::new(resampled, channels, DEFAULT_BASE_FREQ).with_wavetable(
            wavetable_cycle_len(path, Some(source_frames)),
            resample_ratio,
        ),
    )
}

/// Serum's cycle length, and the de facto standard for wavetable packs.
const SERUM_CYCLE: u32 = 2048;

/// Bytes of a file header searched for the `clm ` chunk. The chunk sits with
/// `fmt ` ahead of `data` in every exporter that writes one.
const HEADER_SCAN_BYTES: u64 = 4096;

/// Cycle length a WAV declares for itself, in source-file samples.
///
/// Serum-family exporters write a RIFF `clm ` chunk whose payload reads
/// `<!>2048 20000000 wavetable (www.xferrecords.com)`. Anything else, including
/// an unreadable file or a non-WAV, is `None`.
fn declared_cycle_len(path: &Path) -> Option<u32> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
    {
        return None;
    }

    let mut head = Vec::new();
    File::open(path)
        .ok()?
        .take(HEADER_SCAN_BYTES)
        .read_to_end(&mut head)
        .ok()?;

    if head.len() < 12 || &head[0..4] != b"RIFF" || &head[8..12] != b"WAVE" {
        return None;
    }

    let mut pos = 12;
    while pos + 8 <= head.len() {
        let size = u32::from_le_bytes(head[pos + 4..pos + 8].try_into().ok()?) as usize;
        let body = pos + 8;
        if &head[pos..pos + 4] == b"clm " {
            let end = body.saturating_add(size).min(head.len());
            return parse_clm(&head[body..end]);
        }
        // Chunk bodies are padded to an even length.
        pos = body.saturating_add(size).saturating_add(size & 1);
    }
    None
}

/// Reads the leading integer out of a `clm ` payload.
fn parse_clm(body: &[u8]) -> Option<u32> {
    let digits = std::str::from_utf8(body).ok()?.strip_prefix("<!>")?;
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse().ok()
}

/// Wavetable cycle length for a file, in source-file samples, `0` when the file
/// is not a wavetable.
///
/// The file's own declaration wins. Without one, a length that divides evenly
/// into [`SERUM_CYCLE`] frames is taken as a pack that simply omitted the chunk.
/// A file exactly one cycle long stays `0`: the whole buffer is the cycle.
///
/// `source_frames` must be the length of the whole original file. A decoded
/// length will not do: [`decode_sample_head`] truncates to [`HEAD_FRAMES`],
/// itself a multiple of [`SERUM_CYCLE`], so the fallback would tag every long
/// sample in the library as a wavetable. `None` declines the fallback.
fn wavetable_cycle_len(path: &Path, source_frames: Option<u32>) -> u32 {
    if let Some(declared) = declared_cycle_len(path) {
        if declared >= 2 && source_frames.is_none_or(|frames| declared <= frames) {
            return declared;
        }
    }
    match source_frames {
        Some(frames) if frames > SERUM_CYCLE && frames % SERUM_CYCLE == 0 => SERUM_CYCLE,
        _ => 0,
    }
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
    let channels = codec_params.channels.map(|c| c.count().max(1)).unwrap_or(1) as u8;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100) as f32;
    let file_n_frames = codec_params.n_frames;
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

    let resample = (sample_rate - target_sr).abs() > 1.0;
    let resample_ratio = if resample {
        target_sr / sample_rate
    } else {
        1.0
    };
    let resampled = if resample {
        resample_linear(&samples, channels as usize, sample_rate, target_sr)
    } else {
        samples
    };

    let decoded_frames = (resampled.len() / channels as usize) as u32;
    let total_frames = match file_n_frames {
        Some(n) => {
            let n = if resample {
                (n as f32 * resample_ratio) as u32
            } else {
                n as u32
            };
            n.max(decoded_frames)
        }
        None => decoded_frames,
    };

    // Only the whole file's length can drive cycle detection; the head we just
    // decoded is truncated to a multiple of SERUM_CYCLE by construction.
    let source_frames = file_n_frames.map(|n| n as u32);

    Ok(
        SampleData::new_head(resampled, channels, DEFAULT_BASE_FREQ, total_frames).with_wavetable(
            wavetable_cycle_len(path, source_frames),
            resample_ratio,
        ),
    )
}

/// Resamples interleaved audio using linear interpolation.
///
/// Simple but fast resampling suitable for non-critical applications.
/// For higher quality, consider using a dedicated resampling library like rubato.
pub(crate) fn resample_linear(
    samples: &[f32],
    channels: usize,
    from_sr: f32,
    to_sr: f32,
) -> Vec<f32> {
    let ratio = to_sr / from_sr;
    let in_frames = samples.len() / channels;
    let out_frames = (in_frames as f32 * ratio) as usize;
    let mut output = vec![0.0; out_frames * channels];

    for out_frame in 0..out_frames {
        let in_pos = out_frame as f32 / ratio;
        let in_frame = (in_pos as usize).min(in_frames.saturating_sub(1));
        let next_frame = (in_frame + 1).min(in_frames.saturating_sub(1));
        let frac = in_pos - in_frame as f32;

        for ch in 0..channels {
            let s0 = samples[in_frame * channels + ch];
            let s1 = samples[next_frame * channels + ch];
            output[out_frame * channels + ch] = s0 + frac * (s1 - s0);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Builds a minimal WAV header, optionally carrying a `clm ` chunk.
    fn wav_header(clm: Option<&str>) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&0u32.to_le_bytes()); // size, unread
        out.extend_from_slice(b"WAVE");

        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&[0; 16]);

        if let Some(text) = clm {
            out.extend_from_slice(b"clm ");
            out.extend_from_slice(&(text.len() as u32).to_le_bytes());
            out.extend_from_slice(text.as_bytes());
            if text.len() % 2 == 1 {
                out.push(0);
            }
        }

        out.extend_from_slice(b"data");
        out.extend_from_slice(&0u32.to_le_bytes());
        out
    }

    fn write_temp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut f = File::create(&path).expect("temp file");
        f.write_all(bytes).expect("write temp file");
        path
    }

    #[test]
    fn clm_chunk_yields_cycle_length() {
        let path = write_temp(
            "doux_clm_serum.wav",
            &wav_header(Some("<!>2048 20000000 wavetable (www.xferrecords.com)")),
        );
        assert_eq!(declared_cycle_len(&path), Some(2048));
    }

    #[test]
    fn wav_without_clm_chunk_declares_nothing() {
        let path = write_temp("doux_clm_absent.wav", &wav_header(None));
        assert_eq!(declared_cycle_len(&path), None);
    }

    #[test]
    fn malformed_clm_payloads_do_not_panic() {
        for (name, payload) in [
            ("doux_clm_empty.wav", ""),
            ("doux_clm_noprefix.wav", "2048 wavetable"),
            ("doux_clm_nodigits.wav", "<!>wavetable"),
            ("doux_clm_huge.wav", "<!>99999999999999999999"),
        ] {
            let path = write_temp(name, &wav_header(Some(payload)));
            assert_eq!(declared_cycle_len(&path), None, "payload {payload:?}");
        }
    }

    #[test]
    fn truncated_header_declares_nothing() {
        let full = wav_header(Some("<!>2048 wavetable"));
        let path = write_temp("doux_clm_truncated.wav", &full[..20]);
        assert_eq!(declared_cycle_len(&path), None);
    }

    #[test]
    fn non_wav_extension_is_skipped() {
        let path = write_temp("doux_clm_wrong_ext.mp3", &wav_header(Some("<!>2048 x")));
        assert_eq!(declared_cycle_len(&path), None);
    }

    #[test]
    fn undeclared_pack_falls_back_to_serum_cycle() {
        let path = write_temp("doux_fallback.wav", &wav_header(None));
        assert_eq!(wavetable_cycle_len(&path, Some(16 * SERUM_CYCLE)), SERUM_CYCLE);
    }

    #[test]
    fn single_cycle_file_stays_undeclared() {
        let path = write_temp("doux_single_cycle.wav", &wav_header(None));
        // Exactly one cycle long, and any length that is not a whole number of
        // cycles, both mean "the whole buffer is the cycle".
        assert_eq!(wavetable_cycle_len(&path, Some(SERUM_CYCLE)), 0);
        assert_eq!(wavetable_cycle_len(&path, Some(600)), 0);
    }

    #[test]
    fn declaration_longer_than_the_file_is_rejected() {
        let path = write_temp("doux_clm_too_long.wav", &wav_header(Some("<!>2048 x")));
        assert_eq!(wavetable_cycle_len(&path, Some(512)), 0);
    }

    #[test]
    fn cycle_frames_scales_by_resampling() {
        let data =
            SampleData::new(vec![0.0; 8192], 1, 261.626).with_wavetable(2048, 48000.0 / 44100.0);
        // 2048 source samples become 2048 * 48000/44100 stored frames.
        assert!((data.cycle_frames(0) - 2229.12).abs() < 0.1);
        // A caller override is quoted in source-file samples too.
        assert!((data.cycle_frames(1024) - 1114.56).abs() < 0.1);
    }

    #[test]
    fn cycle_frames_falls_back_to_the_whole_buffer() {
        let data = SampleData::new(vec![0.0; 600], 1, 261.626);
        assert_eq!(data.cycle_frames(0), 600.0);
    }
}

