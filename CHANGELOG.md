# Changelog

All notable changes to doux are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- **Granular sample reader**: `grain` (grain size in ms, 0 = off, modulatable) switches `Source::Sample` from the phase vocoder to a cloud of short Hann-windowed grains. `spray` (0-1) scatters each grain's start position within the `begin`/`end` region and its stereo placement by the same amount, `dens` (1-8, default 2) sets how many overlap. `stretch` keeps its exact meaning and drives the cloud's scan head instead of the vocoder, `speed`/`note` pitch the grains, so `grain` selects the algorithm rather than adding a second time control
- The granular branch is tested first in the `Source::Sample` arm and early-returns, so an unused `grain` costs one `f32` compare per block. `GrainState` is ~260 bytes and lives inline on `Voice`, not boxed like `StretchState`, and shares the phase vocoder's pre-warmed Hann table
- Grain placement uses the square-root law `sqrt(2(1-p))` / `sqrt(2p)`, which is equal-power and exactly unity at the centre, so `spray/0` is bit-identical to no placement. Note `spread` has never applied to samples: it gates on `nch == 1` and the sample source is stereo
- Grain slots are allocated by stealing the most-faded grain, the same policy `steal_voice_slot` applies to voices. Round-robin is safe only at a fixed `grain`: modulating it downward raises the launch rate while long grains are still fading, and the allocator would reclaim one mid-window, cutting a Hann peak to zero

### Known limitations

- A fractional `n` (the A/B sample crossfade) is ignored in grain mode; the first sample is granulated

## [0.0.44] - 2026-07-27

### Added

- **Master output level** — `Engine::set_master_gain(g)` / `master_gain()`, linear, clamped to `0..=MASTER_GAIN_MAX` (2.0, ~+6 dB). Applied before the limiter's peak detector and ramped across the chunk
- **Limiter metering** — `EngineMetrics::limiter_gr()` reports the master limiter's peak gain reduction over the last block (0..1)
- **`comp` is a real compressor** — new `compthresh`/`cthresh` (dB, default -20) and `compratio`/`cratio` (default 4), both modulatable. Feed-forward gain law `(env/t)^(1/ratio - 1)` above threshold, unity below; `comp` is the dry/wet on that gain. Previously threshold-less, so it ducked at every level
- **Bass mono** — `Engine::set_bass_mono(hz)` / `bass_mono()` collapse the stereo image below a crossover corner (0 = off, capped at 300 Hz). Runs first in the master chain, per output pair
- `distortasym`/`dasym` is now modulatable

### Changed

- **[BREAKING]** `patch/<name>` is a serial orbit insert, not a parallel send — the graph's output replaces the bus, so an orbit patch can subtract and a filter/gate/saturator does what its name says. `patchlevel`/`plevel` becomes the dry/wet mix (0 = untouched bus, 1 = patch alone) instead of a send level
- **[BREAKING]** `comporbit` defaults to the orbit itself, not orbit 0, so a bare `comp` glues its own bus. Negative value resets to self. `Orbit::comp_orbit` is `Option<usize>`, `Event::comporbit` is `Option<Option<usize>>`
- **[BREAKING]** `Orbit::set_mod` returns `bool` (chain installed or not) and is `#[must_use]`
- Voice stealing ranks releasing/dead voices first, then established, then still-attacking, envelope value breaking ties — a burst at the ceiling no longer eats its own newest notes

### Fixed

- **SVF resonance no longer changes loudness** — `svf`/`svf24` rebuilt on a TPT core with a saturated resonance path, input pre-scaled by `1 - 0.5q`, Q curve remapped to `0.5 + 29.5·q^2.5` (the linear `fi.svf` wrapper passed Q=1500 by `q/0.9`, making the top third unusable). Restores the pre-Faust level discipline
- **`comp` above 1 inverted the bus** — the dry/wet gain was only floor-clamped, so `2 comp` gave a phase-inverted, amplifying "compressor". Now clamped to unit range, and the static event path writes through the same `write_param` as the modulation path (also fixed `compratio`, `patchlevel`)
- **Limiter readout missed events** — `limiter_gr` was stored once per block and polled less often; it now accumulates a maximum and clears on read (`take_limiter_gr`)
- **Room-routed orbits lost their compressor and recorder** — the final mix skipped room orbits wholesale, so `comp` stopped working the moment a `superpan` voice latched the orbit into room mode. The gain law is now shared by the pair pass and the room spread
- **Orbit modulation past the cap was silent** — the 17th distinct modulated param on an orbit was dropped with no signal; refusals are now counted in `EngineMetrics::dropped_orbit_mods`

## [0.0.43] - 2026-07-11

### Added

- **Arf patch graphs**: user-defined DSP graphs installed by name, used as sources or effects. A host builds an `arf` graph, serialises it to JSON, installs it into the engine's `PatchRegistry`. `doux` compiles it at the device sample rate and owns polyphony (a per-patch `Vm` pool). `doux` never parses a patch language. The graph JSON is the whole boundary. New `arf` workspace crate, plus `pub mod patch` (`PatchRegistry`, `Engine::patch_registry()`).
  - **As a source**: trigger an input-less graph with `s/arf:<name>`. Gate, note frequency and velocity go into the graph's control lanes per sample. Patches get the same vibrato, glide and mod-chain behaviour as native sources (`Source::Arf`).
  - **As a voice insert**: `fx/<name>` inserts an input-reading graph into a voice's chain, serial, just before the envelope VCA. `fx/off` clears the slot.
  - **As an orbit send**: `patch/<name>` attaches a graph as a parallel send at the end of the orbit FX chain, so it hears the built-in tails (comb → feedback → delay → reverb → patch). Sticky on the orbit. `patch/off` clears it. Send level is `patchlevel`/`plevel` (modulatable).
  - **Named patch params**: address a graph's `param` lanes with `p:<name>/<value>` event keys. Values are static floats or full modulation chains. They ride the same per-sample ParamMod machinery as native params (e.g. `p:cutoff/2000/p:res/0.5~0.9:2`).
  - Source vs. effect role comes from the graph's input channel count, enforced at each use-site. `off` is a reserved patch name, rejected at install.
- `Engine::set_tempo(bps)`: sets the tempo patches read as `bps`, latched every block. Defaults to `2.0` (120 BPM).
- `EngineConfig::patch_registry: Option<Arc<PatchRegistry>>`: lets a host share or persist a registry across engine rebuilds, like `sample_registry`.
- REPL `.patch <name> <graph json>`: compiles and installs a patch, playable as `s/arf:<name>`.
- `PatchRegistry::with_polyphony`/`remove`: build a registry with a chosen Vm-pool size, or evict an installed patch. A voice still sounding keeps its own `Arc` and plays out.
- Website doc "Arf Patches" (`patch`, `patchlevel`, `fx`).

### Changed

- **[BREAKING]** `EngineConfig` gained a required `patch_registry` field. Every `EngineConfig { .. }` literal must now set it.
- `arf` is now a workspace member and a required dependency. `arc_swap` moved from native-only optional to an unconditional dependency (the registry uses it on wasm too).
- **Voice stealing at the polyphony ceiling**: a new note at max voices steals the quietest voice (min envelope, click-free takeover) instead of being dropped. Works on native and wasm. Native still honours the load gate. Events that used to be silently dropped now sound.
- Default per-patch `Vm` pool raised from 8 to `max_voices` (64), so an arf source isn't more polyphony-limited than a native voice.
- **Memory-layout optimization**: off-hot-path heap moves shrink the `Voice` struct and cut per-sample work. The pitch-shifter DSP, time-stretch state and per-orbit block scratch are now boxed. A detune ratio cache runs `exp2f` once per value, not per sample. Modulation runs block-rate unless a stage touches a modulated param. Faust block scratch only initialises the used `[..n]`, not the whole block. Output is bit-identical.
- **[BREAKING]** `schedule::Schedule::push` now returns `Option<Event>` (the rejected event at capacity) instead of dropping it silently. It's `#[must_use]`.

### Fixed

- **RT-safety, event frees off the audio thread**: spent and rejected events go to a `doux-event-reaper` thread. Their `String`/`Vec` fields are freed off the real-time thread (native).
- **Numeric-safety hardening across the DSP**: pathological params can no longer mint NaN/Inf.
  - Every Faust effect clamps its user- and mod-controllable freqs, Q, feedback and windows inside the `.dsp` (SVF/SVF24 resonance, wah, phaser, smear, pitch-shift window, comb/feedback damping, flanger feedback, EQ band freqs and mid-Q). An out-of-range value can no longer divide by ~0 or drive a recursion to NaN.
  - The master output flushes non-finite samples to 0 before the DC-blocker. Before, one NaN latched the blocker and silenced the engine until restart.
  - The compressor sidechain duck-gain base is clamped to `≥ 0` before `powf`. An over-unity sidechain env used to mint a NaN into the master mix.
  - Event and modulation parsing reject `nan`/`inf` tokens. A NaN period slipped past the `<= 0` guard and latched the param dead. `midi2freq`'s octave exponent is clamped, so a huge finite note stays finite.
  - The orbit FX chain recovers if block energy goes non-finite. A patch `Vm` that latches non-finite self-heals by swapping in a fresh pooled `Vm`. Sticky user params survive. Sample decoders guard 0-channel and non-finite or ≤ 0 base frequency.
- **Denormal / frozen-tail flush**: orbit FX tails (delay lines, reverb tanks, comb, feedback, compressor env) are zeroed at the silence-holdoff crossing (< −140 dB, inaudible) and on panic. This bounds the wasm per-sample denormal window.

## [0.0.42] - 2026-06-30

### Added

- VinylSim character insert — `vinyl` (dry/wet), `vinylwow` (wow+flutter), `vinylnoise` (hiss), `vinyltone` (tilt), `vinyltype` (voicing: `dull`/`clear`/`cassette`). Wow/flutter pitch wobble, band-limiting, tape/vinyl hiss + gentle saturation; runs after the distortion group so hiss tracks the note envelope
- Auto-Wah — `wah` (dry/wet), `wahpeak` (resonance), `wahsens` (envelope sensitivity), `wahmanual` (base cutoff Hz). Envelope-follower resonant bandpass; the sweep tracks the live signal
- `distortmode`/`dmode` — saturator curve: `soft` (default, unchanged), `tanh`, `arctan`, `hardclip`, `parabolic`, `sinarctan` (ADAA-antialiased shapers)
- `distortasym`/`dasym` — pre-shaper bias for asymmetric / even-harmonic colour; the induced DC is removed downstream
- `foldmode`/`fmode` — wavefold shape: `triangle` (default, unchanged), `sine`, `wrap`
- `chorustype`/`ctype` — chorus voicing: `classic` (default), `ensemble`, `dimension`
- `flangermode`/`flmode` — `classic` (default) or `throughzero`
- `eqmidq` — mid-peak Q / bandwidth (`0.7` = original bell)
- Frequency shifter — `fshift`/`fsh` (single-sideband shift in Hz, signed: positive shifts up, negative down). Inharmonic, Faust-based
- Pitch shifter — `pshift`/`psh` (granular transposition in semitones, signed), `pshiftwin`/`pwin` (grain window in ms). Faust-based
- Soundfont (SF2/GM) overhaul — exclusive-class drum choke (a note in a non-zero class silences other voices of the same class on the same orbit, e.g. open/closed hi-hat), loop-until-release (SF2 sample mode 3), per-zone vibrato LFO, the SF2 concave velocity→amplitude curve, and stereo L/R sample linking. Samples now play at their native rate — the device-rate ratio folds into playback speed instead of an up-front resample
- GM preset selectable via `n` — bare `gm` takes the preset name/number in `n` (`gm snd piano n`), alongside the inline `gmpiano`/`gmdrums` form (bare `gm` still defaults to program 0)

### Changed

- **Faust DSP rewrite** — the per-voice insert + filter chain and most orbit effects are reimplemented in Faust (`dsp/*.dsp`, compiled via rust-faust to `src/effects/faust_dsp/*_gen.rs`, regenerated by `dsp/regen.sh`). Covers SVF (12/24 dB) + Moog ladder filters; `distort`/`fold`/`crush`/`coarse`; `eq`/`tilt`; `chorus`/`flanger`/`phaser`/`smear`/`haas`; `comb`/`feedback`/`delay` (standard/pingpong/multitap/tape); and the reverb. Native + WASM
- **[BREAKING]** `verbtype`/`vtype` reverb voicing renamed: `plate`/`dattorro` → `cloud`/`jpverb` (now a JPverb-style algorithm). `space`/`vital` unchanged
- **[BREAKING]** Soundfont engine API — `install_soundfont` removed; the GM bank now lives behind an `ArcSwap`, owns its sample PCM (no longer inserted into the global sample registry), and is published atomically. `gm_bank`/`set_gm_bank`/`gm_bank_handle`/`take_gm_bank` take/return `Arc<GmBank>` through `&self` (interior mutability), no longer `&mut self`

### Removed

- **[BREAKING]** Feedback delay LFO params `fblfo`/`fblfodepth`/`fblfoshape` — dropped in the Faust feedback rewrite
- Native modules superseded by Faust: `dsp/filter.rs`, `effects/lag.rs`, `effects/haas.rs`, `effects/vital_reverb.rs` (the `haas` word is unchanged, now Faust-backed)

### Fixed

- Reverb output gain

## [0.0.41] - 2026-06-21

### Fixed

- Output channel count — `--channels` is now honoured on PipeWire/JACK by probing the device (opening a stream) instead of trusting `supported_output_configs()`, which under-reports as stereo. Falls back to the device default on real hardware refusal; rejects 0 channels

## [0.0.40] - 2026-06-11

### Added

- Per-source semantic names for the three generic tone params (`timbre`/`harmonics`/`morph`). New names:
  - `pluck`: `bright` (damping), `ring` (sustain), `excite` (excitation color)
  - `kick`: `drive`, `punch` (sweep speed), `sweep` (sweep depth)
  - `snare`: `snappy` (body/noise mix), `bright`
  - `hat`: `reso`, `bright`, `metal` (ratio spread)
  - `tom`: `noise` (stick noise), `punch`, `sweep`
  - `rim`: `ring` (ring length), `bright`, `shift` (upper partial shift)
  - `cowbell`: `drive`, `bright`, `clang` (detune)
  - `cymbal`: `sizzle` (noise tail), `bright`, `metal`
- `pluck` (aliases `ks`, `string`) — Karplus-Strong plucked string. `bright` = brightness/damping, `ring` = sustain, `excite` = excitation color. Delay retuned per sample, so vibrato and freq modulation bend the string continuously
- Live-voice param addressing: a voice-addressed event without `s`/`sound` retargets params on the sounding voice — no envelope/gate/phase/sample-position touch (drone sculpting: `voice/0/lpf/800`)
- `glide` — portamento time in seconds, sticky on the voice. A static `freq`/`note` retarget on a sounding voice slews from the current pitch; update + glide = legato, retrigger + glide = portamento
- Static values displace modulation: a bare `lpf/800` clears an active ModChain on that param (previously inaudible — the chain kept rewriting the param)

### Changed

- `tri` now has polyBLAMP corner anti-aliasing (less sheen at high notes); skipped when `warp`/`mirror`/`size` shaping is active, since shaping moves the corners
- Wavetable `scan` blends cycles with a smoothstep crossfade instead of linear — no zipper when scan modulation crosses cycle boundaries
- **[BREAKING]** `voice/N` is now a stable identity tag, not a slot index (slot indices silently changed when other voices were freed)
- **[BREAKING]** `s` + `voice/N` on a sounding voice retriggers: envelopes re-fire click-free from their current value, gate restarts; params stay sticky (event fields overwrite, no reset-to-defaults). `reset/1` keeps full-reset semantics
- **[BREAKING]** a voice-addressed event for a non-sounding tag without `s` is dropped (previously spawned a default triangle voice)
- `release` command releases the voice by tag
- Live updates no longer stomp unrelated envelope stages with defaults, and no longer clear `inchan` when absent from the event

### Removed

- **[BREAKING]** `add` additive oscillator and its `partials` param — not interesting enough to keep

### Fixed

- WASM: `doux_init` overflowed the default 1MB shadow stack (engine construction temporaries grew with the comb rework) and trapped with "index out of bounds". Orbit array is now heap-built; wasm builds get a 4MB stack

## [0.0.39] - 2026-06-06

### Changed

- Drums reworked for more punch
- `distort` is now a stateful soft-knee saturator
- `fold` — sine wavefolder → reflective triangle wavefolder;
- `phaser` two-stage notch filter → 6-stage allpass cascade with feedback resonance; `phaserdepth` now sets feedback resonance.
- `chorus` — base delay one-pole smoothed (no clicks when `chorusdelay` jumps)
- `wrap` — baked pre-wrap DC bias for even-harmonic thickness
- SVF resonance curve steepened

### Removed

- **[BREAKING]** `distort` free function — replaced by the `Saturate` effect

## [0.0.38] - 2026-05-27

### Added

- `superpan`/`span` — equal-power azimuth panning (SuperCollider `PanAz`-style) over a ring of output pairs, for multi-speaker setups
- `superwidth`/`swidth` — number of adjacent output pairs lit (~2 = localised, larger spreads the source wider)
- `speakers`/`spk` — ordered, 1-based output-pair selection (e.g. `1,3,5,7`); empty = all pairs in order
- `superpan` and `superwidth` are modulation targets
- Orbit room-routing: a `superpan` voice's dry stays off the orbit's stereo pair; the wet-only FX return is routed to the room and latched so tails keep flowing after the source stops

## [0.0.37] - 2026-05-26

### Added

- `EngineConfig` + `native`/`wasm` builders; `Engine::new(EngineConfig)` the single constructor
- Engine accessors for config, voice counts, metrics, sample registry
- Soundfont API: `install_soundfont`, `gm_bank`, `set_gm_bank`, `take_gm_bank`
- `dsp::ftz()` / `enable_flush_to_zero()` — FTZ/DAZ on the audio thread, avoids denormal CPU spikes
- Recording-state fields on `EngineMetrics` for host UIs (`rec_active`, `rec_orbit`, `rec_elapsed_frames`, `rec_name`, …). Native only
- `Event::rec_stop` + `endrec` parse key — explicit recording stop
- `types` block-size/queue constants; `pub use arc_swap`

### Changed

- **[BREAKING]** `Engine::new(EngineConfig)` replaces `new_with_channels`/`new_with_metrics`
- **[BREAKING]** `load_soundfont_from_dir` → `install_soundfont`
- **[BREAKING]** `types` consts renamed (`WASM_BLOCK_SIZE`→`WASM_BUFFER_SIZE`, `DEFAULT_NATIVE_BLOCK_SIZE`→`DEFAULT_BUFFER_SIZE`)
- **[BREAKING]** Recording stop is explicit (`/doux/rec/endrec/1`); `/doux/rec/{name}` starts, nameless `/doux/rec` is a no-op
- Block-rate voice kernel: voices run in inner DSP blocks decoupled from host buffer; voice chain rebuilt as a Stage program
- Engine periphery tightened: block-invariant asserts, dead cache dropped, FX buffers scaled to block
- Recorder rewritten around a lock-free SPSC ring + writer thread; overdub mix + finalize off the audio thread
- `Recorder` `toggle_rt` → explicit `start()`/`stop()`
- Recording cap 60 s → ~10 min (writer-side)

### Fixed

- Recording is now real-time-safe: the audio thread no longer allocates. Previously every stop allocated ~23 MB in the callback, causing xruns and JACK/PipeWire crashes on Linux

### Removed

- `Engine::gen_sample`, `Recorder::toggle_rt`, `RecorderWorker`, `RecorderJob`, `RecorderRtResult`
- RT-side recording auto-naming (`recN`)

## [0.0.36] - 2026-05-15

### Added

- OSC bundle NTP timetags resolve to sample-accurate ticks via `TimeAnchor`. In-band `tick`/`time`/`delta` still override; OSC "immediately" `(0, 1)` fires on receipt
- Steep 24 dB/oct SVF variants: `slpf`/`slpq`, `shpf`/`shpq`, `sbpf`/`sbpq` 
- `fmpivot` (continuous 0–1, wraps) replaces `fmalgo`

### Changed

- **[BREAKING]** Orbit FX run as a sequential chain (comb → feedback → delay → reverb), not parallel sends. Reverb now captures delay tails; per-effect send/out buses collapsed to a single orbit bus
- Per-FX param structs (`DelayParams`, `ReverbParams`, `CombParams`, `FeedbackParams`, `CompressorParams`) replace the monolithic `EffectParams` enum. Send levels live on the orbit
- SVF: tanh-bounded bandpass feedback; Q curve remapped to `2·(1−q)^2.5` (was `2·10^(-2q)`); input scaled by `1 − 0.5q` so the resonant peak doesn't grow louder with Q
- REPL "CPU" → "Load" — it's a per-callback time budget ratio (0–2.0), not machine CPU.

### Removed

- Benchmark/profiling harness: `doux-bench`, `src/benchmark.rs`, `benches/engine.rs`, `tests/perf_workflow.rs`, `criterion` dep (~895 LOC)
- `fmalgo` — see `fmpivot`

## [0.0.35] - 2026-05-08

### Added

- Public `all_modules()` exposes the source/effect registry (names, aliases, params, defaults, ranges) for sova docs

### Changed

- Scope capture buffer → 2048 samples

## [0.0.34] - 2026-05-08

### Changed

- Orbit FX params (`feedback`/`fb`, `fbtime`/`fbt`, `comb`, `combfreq`, `combfeedback`, `comp`, `delay`, `delaytime`, `delayfeedback`, `verb`/`reverb`) demoted from per-voice modulatable to orbit-scoped scalars (SuperDirt semantics)
- Orbit silence holdoff: hardcoded `48000` samples → sample-rate-aware `1.0s`
- Drop unused f64 `fast_tanh`; dedupe `PhaseShape::apply_or_pass` across oscillators; simplify `compressor` / `delay` / `sampling`

### Fixed

- Stack overflow on engine init — `Feedback` `[DelayLine<32768>; 2]` per orbit moved off stack to heap `Vec<f32>`

## [0.0.33] - 2026-05-04

### Changed

- 4-tap cubic Hermite interpolation for all sample reads (was linear)
- SVF: cache full coefficient set, drop tanh from integrator state
- Biquad: Direct Form I → Transposed Direct Form II
- Output stage: drop limiter and headroom soft-clip; plain tanh

## [0.0.32] - 2026-04-21

### Added

- Hard sync (`sync`, `syncphase`/`syncph`) and soft sync (`syncmode`) on all basic oscillators, `add`, and `osc`
- Cross-channel feedback blend (`fbcross`/`fbc`) for ping-pong-style per-orbit feedback
- Room-size parameter (`verbsize`/`vsize`) on the space reverb, separated from diffusion
- 909-style click transient and two-stage pitch envelope on kick
- 808-style dual-partial snare with highpassed-noise rattle tail

### Changed

- Removed always-on master saturation
- Removed voice-count gain compensation in favor of fixed per-voice trim plus linked output limiting and soft clipping
- Switched native sample metadata on the callback path from owned `String`/`PathBuf` clones to shared `Arc<str>` / `Arc<PathBuf>` handles
- `distort` now uses linear drive with unbounded amount instead of `exp_m1` mapping over 0..10
- FM synthesis converted to DX7-style phase modulation (carrier frequency untouched, modulator output offsets carrier phase)
- `fold` and `wrap` distortions anti-aliased via first-order antiderivative (ADAA)
- `fbtime` max range raised from 500 ms to 680 ms
- `phasersweep` now expressed in cents instead of Hz; reverb/flanger/chorus/phaser defaults retuned
- Cowbell and cymbal use band-limited square oscillators

### Fixed

- Preserved stereo in orbit comb, feedback, and reverb sends
- Removed the temporary 10x Vital space reverb compensation and corrected the space reverb wet-path scaling
- Removed hot-path heap allocation when scheduled sample events resolve or upgrade native registry samples
- PolyBLEP/BLAMP anti-aliasing around hard-sync reset on saw, and reverse-direction polyBLEP for soft sync
- DC blocker added after the distortion chain to remove asymmetric-drive DC creep

## [0.0.31] - 2026-04-09

### Added

- **Native benchmarking tools** — added a dev-only `doux-bench` binary, a shared offline runner, and a checked-in benchmark corpus for repeatable native engine measurements
- **Internal hotspot profiling** — added optional native phase profiling plus regression coverage for the benchmark and perf-analysis workflow

### Changed

- **Offline render path reuse** — `doux-render`, `doux-bench`, and `cargo bench` now share the same native offline stepping path
- **Cheaper additive and spread rendering** — additive voices now cache partial data per voice instead of rebuilding it every sample, substantially reducing the cost of additive stress cases
- **Lower orbit-routing overhead** — orbit FX params are now collected once per block instead of being rewritten in the per-sample inner loop
- **CLI functions return `Result`** — `resolve_output_config`, `build_audio_streams`, `osc::run`, and `osc::run_recoverable` now return errors instead of panicking on missing devices, bad configs, or port conflicts
- **CLI audio callback panic safety** — `build_output!` in `cli_common.rs` now uses `catch_unwind`, matching the Sova integration path

### Fixed

- **Command channel survives device reconnection** — `reconnect_streams()` no longer creates a new `cmd_tx`/`cmd_rx` pair. The channel is created once in `start()` and reused across reconnections, so the `SovaReceiver` and `EngineWorker` keep working after a device loss/recovery cycle
- **Failed reconnection no longer disables retry** — `device_lost` flag is only cleared after a successful `build_streams()`; previously a failed reconnection cleared the flag and silently gave up

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
