# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- New **space** reverb algorithm based on Vital's reverb design, with per-line T60 decay, pre/post filtering, chorus modulation, and allpass diffusion network

### Performance

- Cached expensive per-sample math in **space** reverb, SVF filters, and **plate** reverb buffers; eliminated heap allocations from event parsing and sample lookup on the audio thread

### Removed

- Removed `ftype` parameter (filter slope cascading). Ladder filters (`llpf`/`lhpf`/`lbpf`) provide steep slopes with proper nonlinear saturation.

### Changed

- Renamed reverb algorithms to user-facing names: "vital" is now **space**, "dattorro" is now **plate**. Old names still work as aliases.

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
