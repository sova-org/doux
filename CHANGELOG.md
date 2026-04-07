# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.0.31] - No planned release date

### Added

- **Native benchmarking tools** — added a dev-only `doux-bench` binary, a shared offline runner, and a checked-in benchmark corpus for repeatable native engine measurements
- **Internal hotspot profiling** — added optional native phase profiling plus regression coverage for the benchmark and perf-analysis workflow

### Changed

- **Offline render path reuse** — `doux-render`, `doux-bench`, and `cargo bench` now share the same native offline stepping path
- **Cheaper additive and spread rendering** — additive voices now cache partial data per voice instead of rebuilding it every sample, substantially reducing the cost of additive stress cases
- **Lower orbit-routing overhead** — orbit FX params are now collected once per block instead of being rewritten in the per-sample inner loop

## [0.0.30] - 2026-04-06

### Added

- **Load shedding** — when DSP load exceeds 95%, the engine hard-cuts voices in release phase first, then force-releases the quietest voices. A load gate (smoothed > 85%) prevents new voice allocation until load recovers
- **Instant load metric** — `ProcessLoadMeasurer` now exposes per-callback instantaneous load alongside the smoothed value

### Changed

- **Effects always pre-allocated** — `flanger`, `chorus`, and `haas` are allocated at voice init instead of lazily on first use. `ensure_effects()` is now a no-op, removing all conditional heap allocation from the audio path
- **Panic safety in audio callback** — the CPAL output callback is wrapped in `catch_unwind`; on panic it latches to silence instead of unwinding through C/ALSA
- **Command drain budget** — audio callback processes at most 64 commands per buffer to bound worst-case latency
- **Larger pre-allocation headroom** — scratch and conversion buffers sized to 8192 samples (was 4096)
- **Faster load smoothing** — smoothing factor lowered from 0.9 to 0.6 for quicker response to load spikes
- **Release profile uses `panic = "unwind"`** — required for `catch_unwind` in the audio callback

### Fixed

- **Voice reset no longer drops effect allocations** — `reset()` reinitializes effects in-place instead of setting them to `None`, preventing re-allocation on reuse

## [0.0.29] - 2026-04-05

### Fixed

- **RT-safety: move event parsing off the audio thread** — `Event::parse()` now runs on the sender thread (`SovaReceiver`), not in the CPAL callback. A new `dispatch_event(Event)` method accepts pre-parsed events, eliminating all String and Vec allocations from the audio path
- **RT-safety: pre-compute effective sample names** — added `effective_name` field to `Event`, computed once during `parse()`. Removes repeated `format!()` and `.clone()` calls in `process_event()` and `update_voice_params()`
- **RT-safety: pre-allocate audio callback buffers** — `conv_buf`, `live_scratch`, and `scratch` are now sized before entering the CPAL closure so `resize()` never allocates on the real-time thread
- **RT-safety: remove heap allocations from voice effects** — `Flanger`, `Chorus`, and `Haas` effects no longer use `get_or_insert_with(Box::new(...))` on the audio thread; effects are guaranteed pre-allocated by `ensure_effects()`
- **RT-safety: remove `.unwrap()` in `process_schedule()`** — replaced with safe `match` to prevent panics on the audio thread
- **RT-safety: remove `orbit_rec_bus.resize()` and `output.resize()` from `process_block()`** — replaced with bounded copies against pre-allocated buffers
- **Fix index-out-of-bounds in scope capture** — changed `chunks()` to `chunks_exact()` preventing a panic when the audio buffer is not evenly divisible by the channel count

## [0.0.28] - 2026-04-04

### Added

- **Slew modulation** — new `>target:duration[curve]` syntax for audio-rate parameter transitions from the current value. Useful for persistent voices (`gate/0` + `voice/N`) where you want smooth parameter changes without specifying the start value

## [0.0.27] - 2026-04-01

### Changed

- **GM soundfont sources are now prefixed** — instead of `sound=gm` + `n=piano`, presets are selected directly as `gmpiano`, `gmtrumpet`, `gmdrums`, etc. Bare `gm` still works (defaults to program 0)

## [0.0.26] - 2026-04-01

### Added

- **Soundfont preset discoverability** — added `gm_preset_docs()` and `SourceDoc` documentation API exposing all GM preset names, aliases, program numbers, and instrument families

## [0.0.25] - 2026-03-31

### Added

- **Better soundfont playback** — attenuation, pan, low-pass filter, scale tuning, and full DAHDSR envelope extracted from SF2 zones

### Fixed

- **SF2 generator resolution** — preset-level generators are now correctly added as offsets instead of used as fallbacks, per SF2 spec

## [0.0.24] - 2026-03-31

### Added

- **`osc` source** — morphing oscillator that sweeps sine → triangle → saw → square via the `wave` parameter (0–1, modulable)

### Fixed

- **doux-sova: move all file I/O off the real-time audio thread** — soundfont loading (SF2 parsing + resampling) and sample directory scanning no longer run inside the CPAL callback. A dedicated `engine-worker` background thread handles heavy I/O and forwards pre-computed results to the audio thread via lightweight channel messages. Fixes "UTC time limit expired" crashes on Linux

## [0.0.23] - 2026-03-29

### Added

- **Audio-rate modulation for `mirror`** — the phase mirror parameter now supports inline modulation syntax (e.g. `mirror/0~1:2`, `mirror/0^0.8:0.01:0.2:0.5:0.3`)

### Changed

- **Less aggressive master output and voice compensation** — removed 0.7 pre-gain from master tanh soft clip, switched voice count compensation from sqrt to cbrt for more dynamic, crispier sound

## [0.0.22] - 2026-03-28


### Changed

- **Lazy-allocated heavy voice effects** — chorus, flanger, and haas delay buffers (~20.5 KB per voice) are now `Option<Box<T>>`, allocated only when the effect is first used. Voices that don't use these effects carry ~1 KB instead of ~21 KB
- **In-place voice reset** — voice reuse on note triggers no longer drops and reallocates the entire struct; `Voice::reset()` resets fields in-place and drops unused effect boxes, eliminating heap churn on the audio thread
- **Faster fastmath via division-free polynomials** — replaced Padé rationals with minimax polynomials in `sinf`, `exp2f`, `log2f`, and `pow10`. Added Coranac weight correction to `par_sinf` for much better accuracy

### Removed

- **`expf` and `expm1f` from fastmath** — slower than std, call sites now use `f32::exp()` and `f32::exp_m1()` directly

## [0.0.21] - 2026-03-26

### Added

- **Audio-rate modulation for 8 new parameters** — `fbtime`, `combfreq`, `combfeedback`, `delaytime`, `delayfeedback`, `eqlofreq`, `eqmidfreq`, `eqhifreq` now support inline modulation syntax (e.g. `delaytime/0.1~0.5:2`, `combfreq/200~2000:4t`)

### Fixed

- **Wavetable scan with modulation** — using modulation syntax on scan (e.g. `scan/0~1:2`) caused the voice to fall back to `Source::Sample`, playing the wavetable linearly instead of scanning between cycles

## [0.0.20] - 2026-03-21

### Fixed

- **doux-sova: cpal 0.17 compatibility** — `SampleRate` changed from tuple struct `SampleRate(u32)` to plain `u32` type alias in cpal 0.17; removed `.0` field access in `negotiate_stream_config()` and stream setup

## [0.0.19] - 2026-03-20

### Added

- Support i32/i16 sample formats at cpal boundary for ASIO compatibility

### Fixed

- Device name matching

## [0.0.17] - 2026-03-19

### Added

- **Host selection in `DouxConfig`** — `host: Option<String>` field lets the GUI explicitly select ASIO vs WASAPI (or JACK vs ALSA). `DouxManager` resolves devices through the selected host instead of hardcoding `preferred_host()`
- **ASIO/JACK buffer size handling** — `host_controls_buffer_size(&Host)` replaces `is_jack_host()`, covering both JACK and ASIO
- **cpal re-export** — `doux::audio::cpal` eliminates `doux-sova`'s direct cpal dependency

## [0.0.16] - 2026-03-19

### Added

- **Per-channel peak metering** — lock-free double-buffered `PeakCapture` accumulates per-channel `max(abs())` from interleaved output, supporting up to 32 channels with no heap allocation in the audio callback. `DouxManager` exposes `peak_capture()` accessor alongside existing `scope_capture()`

## [0.0.15] - 2026-03-18

### Fixed

- **Cut group retrigger** — reuse the matched voice slot in-place instead of allocating a new one, eliminating double-attack transient
- **Cut group amplitude dip** — preserve envelope level across voice reset so retrigger ramps from old level instead of silence
- **Loop boundary double attacks** — `floor()` instead of `round()` in time-to-tick conversion prevents two cycle-boundary events from snapping to the same tick
- **Dropped event counter** — `EngineMetrics.dropped_events` tracks late events silently discarded by the scheduler

## [0.0.14]

### Changed

- **ASIO feature flag** — opt-in `asio` feature enables ASIO backend on Windows. `preferred_host()` tries ASIO first (if a working output device exists), falling back to WASAPI
- **`doux-sova` ASIO forwarding** — `asio = ["doux/asio"]` feature in doux-sova

### Fixed

- **Linux host validation** — `preferred_host()` now verifies the host has a working output device before selecting it, preventing crashes when JACK/PipeWire reports available but can't provide a device
- **Linux device selection** — `default_output_device()` and `default_input_device()` only use JACK client names when JACK is the preferred host, preventing hangs under PipeWire
- **Buffer underrun logging** — `BufferUnderrun` stream errors now logged as xrun in both `cli_common` and `doux-sova` manager
- **Linux diagnostics** — shows host selection reason, checks for `pipewire-alsa` package

## [0.0.13] - 2026-03-14

### Changed

- **Linux host selection** — `HostSelection` now includes `PipeWire` and `PulseAudio` variants. `preferred_host()` priority changed from JACK → ALSA to PipeWire → JACK → ALSA. Requires CPAL 0.18 (blocked on `midir` updating its `alsa` dependency to allow 0.11); on CPAL 0.17 the new variants are accepted but have no effect

## [0.0.12]

### Added

- **Internal parameter metadata** — every source and effect now carries static `ModuleInfo` with parameter names, aliases, descriptions, defaults, and ranges, queryable at runtime via `all_modules()`
- **Envelope modulation type** (`^`) — gate-aware DAHDSR envelope applicable to any modulatable parameter via inline syntax (`min^max:attack:decay:sustain:release`). Replaces per-module filter, pitch, and FM envelopes with a single universal mechanism

### Changed

- **Shared CLI infrastructure** — extracted duplicated device enumeration, stream building, output config resolution, and device-loss recovery from `server.rs` and `repl.rs` into `cli_common` module. Exposed `find_device` from `audio` module
- **`DelayLine` DSP primitive** — extracted circular buffer with linear-interpolated reads into `dsp::DelayLine<N>`, replacing inlined delay logic in chorus, comb, feedback, flanger, and haas effects
- **`AudioCmd` moved to crate root** — extracted from `osc` module to `lib.rs` since it's a general engine command type used by all CLI binaries, not OSC-specific
- **DAHDSR envelope** — replaced ADSR with a six-phase envelope: Delay, Attack, Hold, Decay, Sustain, Release. New `envdelay` (alias `envdly`) and `hold` (alias `hld`) parameters. The envelope is now self-timed via `gate` duration instead of responding to an external gate signal
- **`gate` semantics** — `gate` is now the total note duration in seconds (delay + attack + hold + decay + sustain time). `gate/0` means infinite sustain. Replaces the old `duration` parameter
- **Envelope retrigger** — retriggering during delay phase fades from the current value toward 0, eliminating clicks
- **`MAX_PARAM_MODS`** bumped from 8 to 15 — more room for envelope and modulation chains per voice
- **Transition modulation** (`>`) simplified to single-segment only. Multi-segment chaining removed in favor of the new envelope modulation type

### Removed

- **Mutable Instruments Plaits oscillators** — removed all 10 Plaits synthesis engines (`modal`, `va`, `ws`, `fm2`, `grain`, `additive`, `wavetable`, `chord`, `swarm`, `pnoise`) and the `mi-plaits-dsp` dependency. The native additive oscillator (`add`) retains `harmonics`, `timbre`, `morph`, and `partials` parameters
- **Glide (portamento)** — removed `glide` parameter from engine, event parsing, and documentation. Audio-rate frequency modulation (`freq` with `>`, `~`, `^`) replaces this functionality
- **Repeat** — removed `repeat` parameter from engine, event parsing, and documentation
- **`duration` parameter** — removed in favor of `gate`
- **Per-module filter envelopes** (`lpe/lpa/lpd/lps/lpr`, `hpe/hpa/hpd/hps/hpr`, `bpe/bpa/bpd/bps/bpr`) — use envelope modulation on the cutoff parameter instead (e.g. `lpf/200^8000:0.01:0.1:0.5:0.3`)
- **Pitch envelope** (`penv/patt/pdec/psus/prel`) — use `freq` or `detune` with `^` envelope modulation instead
- **FM envelope** (`fme/fma/fmd/fms/fmr`) — use `fm` with `^` envelope modulation instead (e.g. `fm/0^5:0.01:0.1:0.3:0.5`)

### Fixed

- **BLOCK_SIZE** — clarified how BLOCK_SIZE is used throughout the engine (WASM / native confusion)
- **WASM build** — fixed `WASM_WASM_BLOCK_SIZE` double-prefix typo in `src/wasm.rs`

## [0.0.10] - 2026-03-12

### Added

- **Input channel selection** — `inchan` parameter selects which audio input channel to use for live input (e.g. `inchan/0` for mono from first channel). Defaults to stereo when unset
- **Modulation curves: swell, pluck, stair** — three new transition curve types: swell (`i`, slow start/fast finish), pluck (`o`, fast attack/slow settle), stair (`p`, 8 discrete steps)

### Fixed

- **Deterministic modulation seeds** — each voice now receives a unique random seed from the engine, so consecutive voices with random modulation (`jit`, `drunk`, etc.) produce different patterns instead of identical ones


## [0.0.9] - 2026-03-11

### Added

- **Tweakable EQ frequencies** — `eqlofreq`, `eqmidfreq`, `eqhifreq` parameters for per-voice EQ band frequency control (defaults: 200, 1000, 5000 Hz)
- **Simplified recording syntax** — `/doux/rec/<name>` shorthand for naming recordings directly

### Changed

- **Lock-free audio architecture** — CLI binaries (`doux`, `doux-repl`) no longer wrap the engine in `Arc<Mutex<Engine>>`. The engine is now owned by the audio callback, commands flow via `crossbeam_channel`, and live audio input uses a `ringbuf` SPSC ring buffer. Eliminates mutex contention between audio and control threads
- **REPL metrics read from atomics** — `.voices`, `.time`, `.stats` commands read directly from `Arc<EngineMetrics>` instead of locking the engine. New `time_bits` atomic field exposes engine time without a mutex
- Doux-sova live input fix (similar to Cagire)

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
