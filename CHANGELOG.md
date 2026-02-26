# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

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
