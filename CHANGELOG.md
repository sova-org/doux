# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.0.8] - 2026-03-07

### Added

- **Time stretching** — phase vocoder for independent pitch and time control during sample playback. New `stretch` parameter controls playback duration without affecting pitch. Includes in-place radix-2 FFT, transient detection via spectral flux, and phase locking to spectral peaks

### Changed

- **Engine performance optimizations** — pre-initialized FFT twiddle factors, relative threshold caching on SVF/ladder filter coefficients (skip recalculation on <0.1% delta), power-of-2 delay buffer with bitwise masking, fast math replacements in reverb (`exp2f`/`expf` instead of `powf`), boxed Plaits arrays to shrink Voice struct, pre-block voice gain compensation moved out of hot loop

### Fixed

- **Event delta** now uses `i64` to support negative time deltas, with clamping to prevent underflow

## [0.0.7] - 2026-03-06

### Changed

- **Tick-based event scheduling** — engine timing refactored from floating-point seconds to integer sample ticks (`u64`) for sample-accurate scheduling. `Event.time` → `Event.tick`, `peek_time()` → `peek_tick()`, tolerance calculated in samples. SOVA integration updated with `sync_to_engine_tick()` and `/tick/` command protocol

## [0.0.6] - 2026-03-03

### Added

- **Sample slicing** — `slice` and `pick` parameters for dividing samples into equal segments with wrap-around and negative indexing
- **Sample crossfading** — fractional `n` values blend between adjacent samples (e.g. `n/1.5` crossfades between sample 1 and 2)
- **Modulation on `note` parameter** — `note` now supports mod chains (transitions, oscillation) mapped through `midi2freq`
- **ModChain `map_values`** — transforms modulation target values through an arbitrary function
- **Auto-recovery from audio device disconnection** — `DouxManager` detects stream errors via `device_lost` flag, exposes `needs_reconnect()` and `reconnect_streams()` for consumers to handle reconnection
- **`osc::run_recoverable`** — OSC server variant that returns on device loss instead of blocking forever, enabling reconnection loops
- **doux-sova soundfont feature** — `doux-sova` now exposes an optional `soundfont` feature flag, re-exports `doux::soundfont`
- **`doux-sova` `load_soundfont_from_paths`** — convenience method to scan paths and load the first valid SF2 file

### Changed

- **JACK is now Linux-only** — `cpal` JACK feature is only enabled on `cfg(target_os = "linux")` instead of all platforms, fixing build issues on macOS and Windows
- **CLI binaries refactored for reconnection** — `doux-repl` and `doux-server` extract stream building into restartable functions (`build_repl_streams`, `build_streams`), enabling device hot-swap
- **`doux-sova` uses git dependency** — `sova_core` switched from local path to `git+https://github.com/sova-org/sova`
- **`DouxManager::start` refactored** — stream creation extracted into `build_streams()` method, reused by `reconnect_streams()`
- **`DouxManager::is_running` checks device state** — returns false when `device_lost` flag is set
- **`DouxManager::state()` reports device errors** — populates `error` field with "Audio device disconnected" when flag is set
- **`Event::resolve_range` replaces inline begin/end logic** — single method used by all sample sources (registry, file, web)

## [0.0.5] - 2026-02-27

### Changed

- **Per-orbit voice gain compensation** — each orbit now scales independently (`1/sqrt(n)` per orbit) instead of globally, so voices on one orbit no longer attenuate unrelated orbits

### Fixed

- `resample_linear` re-export gated on `soundfont` feature instead of `native` to silence unused import warning

## [0.0.4] - 2026-02-26

### Added

- **Drum synthesis engine** with 7 sources: `kick`, `snare`, `hat`, `tom`, `rim`, `cowbell`, `cymbal` — percussive envelope defaults, waveform morphing (`wave`), timbral control via `morph`, `harmonics`, `timbre`
- **Additive oscillator** (`add`) — stacks 1–32 sine partials with spectral tilt, even/odd morph, harmonic stretching, phase shaping. New `partials` parameter
- **SoundFont / General MIDI support** (`gm` source) — load SF2 files, zone lookup by program name/number, note, velocity. 80+ named presets. `n` parameter selects program
- **Internal recorder / overdubbing** — `/doux/rec` toggles recording, auto-naming (`rec0`, `rec1`…), manual naming via `/doux/rec/s/<name>`, overdub mode layers on existing buffer, 60s max. Captured samples are immediately playable
- **Sidechain compressor** — ducking/pumping effect. Parameters: `comp` (amount), `compattack`, `comprelease`, `comporbit` (sidechain source orbit)
- **Smear effect** — 12-stage allpass chain for phase-shifted chirps. Parameters: `smear` (mix), `smearfreq` (break frequency), `smearfb` (feedback/resonance)
- **Stereo filter chains** — per-channel SVF and ladder filters for full stereo processing
- **Voice gain compensation** — automatic attenuation based on active voice count (`1/sqrt(n)`) to prevent clipping
- `wave` parameter for drum oscillator waveform (0 sine → 0.5 triangle → 1 sawtooth)
- `expf`, `fast_tanh_f32`, `fast_tan` fast math approximations

### Removed

- Plaits percussion engines (`bass`/`snare`/`hat`) replaced by native drum synthesis

### Changed

- `doux-sova` uses `sova_core` types directly instead of local type definitions
- Sample playback gain increased (0.2 → 0.7) for consistent gain staging across sources
- Plaits output level increased (0.2 → 0.5)
- Ladder filter converted to f32 with fast tanh approximation
- `Event.n` changed from `Option<usize>` to `Option<String>` to support program name selection

### Fixed

- Master output soft clipping in ladder filter (f64 → f32 conversion)
- Space reverb level imbalance (added 10x gain compensation for VitalVerb)
- `exp2f` bounds checking to prevent overflow/underflow
- SVF filter saturation clamping to prevent divergence in high-feedback scenarios
- Rare bug in sample loading

## [0.0.2] - 2026-02-07

### Added

- Audio-rate parameter modulation system with LFO, envelope, random, and sequence chains
- Modulation shapes: sine, triangle, saw, square, hold, random, drunk walk
- Modulation curves: linear, exponential, smooth
- Per-orbit feedback delay with LFO time modulation (`feedback`, `fbtime`, `fbdamp`, `fblfo`, `fblfodepth`, `fblfoshape`)
- Fast math module (`dsp/fastmath`) with SIMD-friendly approximations for `exp2f`, `log2f`, `sinf`, `cosf`, `powf`, `tanh`

### Changed

- Replaced biquad voice filters (lpf/hpf/bpf) with TPT state variable filters for stable audio-rate modulation
- Replaced `tanh()` with fast approximation in ladder filter, removed coefficient cache
- Normalized filter resonance to `[0.0, 1.0]` range
- Normalized `fold` distortion parameter to `[0.0, 1.0]` range
- Removed dedicated scan LFO parameters (`scanlfo`, `scandepth`, `scanshape`) in favor of generic modulation system
- Removed `sova_core` dependency from `doux-sova`, bridge types defined locally

## [0.0.1] - 2026-02-06

Initial versioned release of doux — a software synthesizer engine for live coding.

### Added

- Core DSP engine with configurable polyphony and voice management
- Wavetable oscillators with dynamic modulation
- 3-OP FM synthesis with optional feedback
- Ladder filter
- Sample playback with head preloading, pitch scaling, begin/end/speed control
- DJ-style 3-band EQ, tilt parameter, Haas effect and stereo width
- FDN reverb, delay
- Amplitude-based envelope transitions (click-free)
- CPAL audio backend with JACK support (Linux, macOS, Windows)
- OSC protocol support
- REPL interface (`doux-repl`)
- Offline render mode (`doux-render`)
- WASM build target
- Metrics API with bank/delta events
- Experimental `fit` command
- Companion website with documentation

### Fixed

- Voice swap dropout on voice death
- Linux audio and JACK compatibility
- Sample pitch scaling and playback speed
