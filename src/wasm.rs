//! WebAssembly FFI bindings for browser-based audio.
//!
//! Exposes the doux engine to JavaScript via a C-compatible interface. The host
//! (browser) and WASM module communicate through shared memory buffers.
//!
//! # Memory Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Static Buffers (shared with JS via pointers)                        │
//! ├─────────────────┬───────────────────────────────────────────────────┤
//! │ OUTPUT          │ Audio output buffer (BLOCK_SIZE × CHANNELS f32)   │
//! │ INPUT_BUFFER    │ Live audio input (BLOCK_SIZE × CHANNELS f32)      │
//! │ EVENT_INPUT     │ Command strings from JS (1024 bytes, null-term)   │
//! │ SAMPLE_BUFFER   │ Staging area for sample uploads (16MB of f32)     │
//! │ FRAMEBUFFER     │ Ring buffer for waveform visualization            │
//! └─────────────────┴───────────────────────────────────────────────────┘
//! ```
//!
//! # Typical Usage Flow
//!
//! ```text
//! JS                              WASM
//! ──                              ────
//! 1. doux_init(sr, voices)    →   Create engine
//! 2. get_*_pointer()          →   Get buffer addresses
//! 3. Write command to EVENT_INPUT
//! 4. evaluate()               →   Parse & execute command
//! 5. [Optional] Write samples to SAMPLE_BUFFER
//! 6. load_sample(len, ch, freq) → Add to pool
//! 7. [Optional] Write mic input to INPUT_BUFFER
//! 8. dsp()                    →   Process one block
//! 9. Read OUTPUT              ←   Get audio samples
//! 10. Repeat 3-9 in audio callback
//! ```
//!
//! # Audio Worklet Integration
//!
//! In the browser, this typically runs in an AudioWorkletProcessor:
//! - `dsp()` is called each audio quantum (~128 samples)
//! - Output buffer is copied to the worklet's output
//! - Input buffer receives microphone data for live processing

#![allow(static_mut_refs)]

use crate::types::{Source, BLOCK_SIZE, CHANNELS};
use crate::Engine;

/// Maximum length of command strings from JavaScript.
const EVENT_INPUT_SIZE: usize = 1024;

/// Ring buffer size for waveform visualization (~60fps at 48kHz stereo).
/// Calculation: floor(48000/60) × 2 channels × 4 (double-buffer headroom) = 6400
const FRAMEBUFFER_SIZE: usize = 6400;

// Global engine instance (single-threaded WASM environment)
static mut ENGINE: Option<Engine> = None;

// Shared memory buffers accessible from JavaScript
static mut OUTPUT: [f32; BLOCK_SIZE * CHANNELS] = [0.0; BLOCK_SIZE * CHANNELS];
static mut EVENT_INPUT: [u8; EVENT_INPUT_SIZE] = [0; EVENT_INPUT_SIZE];
static mut FRAMEBUFFER: [f32; FRAMEBUFFER_SIZE] = [0.0; FRAMEBUFFER_SIZE];
static mut FRAME_IDX: i32 = 0;

/// Sample upload staging buffer (16MB = 4M floats).
/// JS decodes audio files and writes f32 samples here before calling `load_sample`.
const SAMPLE_BUFFER_SIZE: usize = 4_194_304;
static mut SAMPLE_BUFFER: [f32; SAMPLE_BUFFER_SIZE] = [0.0; SAMPLE_BUFFER_SIZE];

/// Live audio input buffer (microphone/line-in from Web Audio).
static mut INPUT_BUFFER: [f32; BLOCK_SIZE * CHANNELS] = [0.0; BLOCK_SIZE * CHANNELS];

// =============================================================================
// Lifecycle
// =============================================================================

/// Initializes the audio engine at the given sample rate and max polyphony.
///
/// Must be called once before any other functions.
#[no_mangle]
pub extern "C" fn doux_init(sample_rate: f32, max_voices: usize) {
    unsafe {
        ENGINE = Some(Engine::new_with_channels(
            sample_rate,
            crate::types::CHANNELS,
            max_voices,
        ));
    }
}

// =============================================================================
// Audio Processing
// =============================================================================

/// Processes one block of audio and updates the framebuffer.
///
/// Call this from the AudioWorklet's `process()` method. Reads from
/// `INPUT_BUFFER`, writes to `OUTPUT`, and appends to `FRAMEBUFFER`.
#[no_mangle]
pub extern "C" fn dsp() {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            engine.process_block(&mut OUTPUT, &SAMPLE_BUFFER, &INPUT_BUFFER);

            // Copy to ring buffer for visualization
            let fb_len = FRAMEBUFFER.len() as i32;
            for (i, &sample) in OUTPUT.iter().enumerate() {
                let idx = (FRAME_IDX + i as i32) % fb_len;
                FRAMEBUFFER[idx as usize] = sample;
            }
            FRAME_IDX = (FRAME_IDX + (BLOCK_SIZE * CHANNELS) as i32) % fb_len;
        }
    }
}

// =============================================================================
// Command Interface
// =============================================================================

/// Parses and executes the command string in `EVENT_INPUT`.
///
/// The command should be written as a null-terminated UTF-8 string to the
/// buffer returned by `get_event_input_pointer()`.
///
/// # Returns
///
/// - Sample index if the command triggered a sample load
/// - `-1` on error or for commands that don't return a value
#[no_mangle]
pub extern "C" fn evaluate() -> i32 {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            let len = EVENT_INPUT
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(EVENT_INPUT_SIZE);
            if len == 0 {
                return -1;
            }
            if let Ok(s) = core::str::from_utf8(&EVENT_INPUT[..len]) {
                let result = engine.evaluate(s).map(|i| i as i32).unwrap_or(-1);
                EVENT_INPUT[0] = 0; // Clear for next command
                return result;
            }
        }
        -1
    }
}

// =============================================================================
// Buffer Pointers (for JS interop)
// =============================================================================

/// Returns pointer to the audio output buffer.
#[no_mangle]
pub extern "C" fn get_output_pointer() -> *const f32 {
    unsafe { OUTPUT.as_ptr() }
}

/// Returns the length of the output buffer in samples.
#[no_mangle]
pub extern "C" fn get_output_len() -> usize {
    BLOCK_SIZE * CHANNELS
}

/// Returns mutable pointer to the event input buffer.
///
/// Write null-terminated UTF-8 command strings here, then call `evaluate()`.
#[no_mangle]
pub extern "C" fn get_event_input_pointer() -> *mut u8 {
    unsafe { EVENT_INPUT.as_mut_ptr() }
}

/// Returns mutable pointer to the sample upload staging buffer.
///
/// Write decoded f32 samples here, then call `load_sample()`.
#[no_mangle]
pub extern "C" fn get_sample_buffer_pointer() -> *mut f32 {
    unsafe { SAMPLE_BUFFER.as_mut_ptr() }
}

/// Returns the capacity of the sample buffer in floats.
#[no_mangle]
pub extern "C" fn get_sample_buffer_len() -> usize {
    SAMPLE_BUFFER_SIZE
}

/// Returns mutable pointer to the live audio input buffer.
///
/// Write microphone/line-in samples here before calling `dsp()`.
#[no_mangle]
pub extern "C" fn get_input_buffer_pointer() -> *mut f32 {
    unsafe { INPUT_BUFFER.as_mut_ptr() }
}

/// Returns the length of the input buffer in samples.
#[no_mangle]
pub extern "C" fn get_input_buffer_len() -> usize {
    BLOCK_SIZE * CHANNELS
}

/// Returns pointer to the waveform visualization ring buffer.
#[no_mangle]
pub extern "C" fn get_framebuffer_pointer() -> *const f32 {
    unsafe { FRAMEBUFFER.as_ptr() }
}

/// Returns pointer to the current frame index in the ring buffer.
#[no_mangle]
pub extern "C" fn get_frame_pointer() -> *const i32 {
    unsafe { &FRAME_IDX as *const i32 }
}

// =============================================================================
// Sample Loading
// =============================================================================

/// Loads sample data from the staging buffer into the engine's pool.
///
/// # Parameters
///
/// - `len`: Number of f32 samples in `SAMPLE_BUFFER`
/// - `channels`: Channel count (1 = mono, 2 = stereo)
/// - `freq`: Base frequency for pitch calculations
///
/// # Returns
///
/// Pool index on success, `-1` on failure.
#[no_mangle]
pub extern "C" fn load_sample(len: usize, channels: u8, freq: f32) -> i32 {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            let samples = &SAMPLE_BUFFER[..len.min(SAMPLE_BUFFER_SIZE)];
            match engine.load_sample(samples, channels, freq) {
                Some(idx) => idx as i32,
                None => -1,
            }
        } else {
            -1
        }
    }
}

/// Returns the number of samples loaded in the pool.
#[no_mangle]
pub extern "C" fn get_sample_count() -> usize {
    unsafe {
        if let Some(ref engine) = ENGINE {
            engine.samples.len()
        } else {
            0
        }
    }
}

// =============================================================================
// Engine State
// =============================================================================

/// Returns the current engine time in seconds.
#[no_mangle]
pub extern "C" fn get_time() -> f64 {
    unsafe {
        if let Some(ref engine) = ENGINE {
            engine.time
        } else {
            0.0
        }
    }
}

/// Returns the engine's sample rate.
#[no_mangle]
pub extern "C" fn get_sample_rate() -> f32 {
    unsafe {
        if let Some(ref engine) = ENGINE {
            engine.sr
        } else {
            0.0
        }
    }
}

/// Returns the number of currently active voices.
#[no_mangle]
pub extern "C" fn get_active_voices() -> usize {
    unsafe {
        if let Some(ref engine) = ENGINE {
            engine.active_voices
        } else {
            0
        }
    }
}

/// Fades out all active voices smoothly.
#[no_mangle]
pub extern "C" fn hush() {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            engine.hush();
        }
    }
}

/// Immediately silences all voices (may click).
#[no_mangle]
pub extern "C" fn panic() {
    unsafe {
        if let Some(ref mut engine) = ENGINE {
            engine.panic();
        }
    }
}

// =============================================================================
// Debug Helpers
// =============================================================================

/// Debug: reads a byte from the event input buffer.
#[no_mangle]
pub extern "C" fn debug_event_input_byte(idx: usize) -> u8 {
    unsafe { EVENT_INPUT.get(idx).copied().unwrap_or(255) }
}

/// Debug: returns the source type of a voice as an integer.
///
/// Mapping: Tri=0, Sine=1, Saw=2, ... LiveInput=11, PlModal=12, etc.
/// Returns `-1` if voice index is invalid.
#[no_mangle]
pub extern "C" fn debug_voice_source(voice_idx: usize) -> i32 {
    unsafe {
        if let Some(ref engine) = ENGINE {
            if voice_idx < engine.active_voices {
                match engine.voices[voice_idx].params.sound {
                    Source::Tri => 0,
                    Source::Sine => 1,
                    Source::Saw => 2,
                    Source::Zaw => 3,
                    Source::Pulse => 4,
                    Source::Pulze => 5,
                    Source::Add => 6,
                    Source::White => 7,
                    Source::Pink => 8,
                    Source::Brown => 9,
                    Source::Sample => 10,
                    Source::Wavetable => 11,
                    Source::WebSample => 12,
                    Source::LiveInput => 13,
                    Source::PlModal => 14,
                    Source::PlVa => 15,
                    Source::PlWs => 16,
                    Source::PlFm => 17,
                    Source::PlGrain => 18,
                    Source::PlAdd => 19,
                    Source::PlWt => 20,
                    Source::PlChord => 21,
                    Source::PlSwarm => 22,
                    Source::PlNoise => 23,
                    Source::Kick => 24,
                    Source::Snare => 25,
                    Source::Hat => 26,
                    Source::Tom => 27,
                    Source::Rim => 29,
                    Source::Cowbell => 30,
                    Source::Cymbal => 31,
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

/// Debug: returns 1 if voice has a web sample attached, 0 otherwise.
#[no_mangle]
pub extern "C" fn debug_voice_has_web_sample(voice_idx: usize) -> i32 {
    unsafe {
        if let Some(ref engine) = ENGINE {
            if voice_idx < engine.active_voices {
                if engine.voices[voice_idx].web_sample.is_some() {
                    1
                } else {
                    0
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}
