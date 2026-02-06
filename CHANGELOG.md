# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Changed

- Replaced biquad voice filters (lpf/hpf/bpf) with TPT state variable filters for stable audio-rate modulation
- Replaced `tanh()` with fast approximation in ladder filter, removed coefficient cache
- Upgraded `cpal` from 0.15 to 0.17
- Renamed `loader` module to `sampling`
- Simplified `clear_samples()` internals

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
