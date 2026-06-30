# Doux

*A software-synthesizer engine for live coding.*

Doux is a real-time synthesis and sampling engine for live coding, written in Rust. It began as a port of [Dough](https://dough.strudel.cc/), the C live-coding audio engine by Felix Roos and contributors ([source](https://codeberg.org/uzu/dough)). The engine is built around a fixed voice architecture. Every voice runs the same chain of oscillators, samplers, filters, and effects. You shape the sound by setting parameters rather than by wiring modules together. Most of that DSP (filters, effects, etc) is written in [Faust](https://faust.grame.fr) and compiled ahead of time to Rust. All control happens over OSC, so any client that can send an OSC message can play it. The same core compiles two ways. The native build is a library and a set of command-line binaries backed by `cpal`, and the `wasm32` build is a module that runs in the browser through an `AudioWorklet`. In both cases Doux is meant to sit inside a larger audio application rather than to stand on its own. There is a documentation and a live playground are at **[doux.livecoding.fr](https://doux.livecoding.fr)**.

## Quickstart

You play Doux by sending it events. Each event is a path of `key/value` pairs: `s/saw/note/60`. The same event can reach the engine three ways.

### OSC Server

The **OSC server** is the usual route. Start `doux`, then send it OSC messages from any client. The message address is ignored, and the arguments are read as alternating string keys and values, which the engine joins into the path described above.

```bash
doux # listens for OSC on UDP 57120
```

```text
# any OSC client → send the arguments:  "s" "saw" "note" 60 "gain" 0.8
# → engine path:  s/saw/note/60/gain/0.8
```

### REPL (Read-Eval-Print-Loop)

The **REPL** is the quickest way to try an idea. Run `doux-repl` and type an event at the prompt to hear it straight away. This is usually very good for testing new stuff.

```text
$ doux-repl
doux> s/saw/note/60      # sawtooth at middle C
doux> s/kick             # kick drum
doux> .hush              # fade everything out
```

### Offline rendering

**Offline rendering** writes directly to a WAV file and needs no audio device, which makes it useful for tests and for bouncing a phrase to disk.

```bash
doux-render -d 2 -e "s/saw/note/60" -o out.wav
```

A handful of sources need no samples at all: the oscillators `sine`, `saw`, `tri`, `pulse`, and `pluck`, and the drum voices `kick`, `snare`, `hat`, `tom`, `rim`, `cowbell`, and `cymbal`. To play your own audio, point `--samples` at a directory and address files by folder and index, as in `s/<folder>/n/<index>`. The full list of parameters, filters, and effects lives at [doux.livecoding.fr](https://doux.livecoding.fr).

## Build

Doux is not published to any package registry, so you build it from source with a stable Rust toolchain.

```bash
cargo build --release                       # builds doux and doux-repl (native, default)
cargo build --release --features render     # also builds doux-render
cargo build --release --features soundfont  # SF2 / General MIDI support
```

Which binaries and capabilities you get depends on the Cargo features you enable.

| Feature | Default | Description |
|---------|---------|-------------|
| `native` | yes | cpal audio, OSC, sample decoding — the `doux` and `doux-repl` binaries |
| `render` | no | the offline `doux-render` binary (WAV output) |
| `soundfont` | no | SF2 / General MIDI playback (the `gm` source) |
| `asio` | no | ASIO host (Windows) |
| `profiling` | no | per-phase DSP timing on stderr |

### Faust DSP

Most of Doux's DSP — the filters and the per-voice and orbit effects — is written in [Faust](https://faust.grame.fr). Each Faust source in `dsp/*.dsp` is compiled ahead of time into a Rust module under `src/effects/faust_dsp/*_gen.rs`. Those generated modules are committed to the repository, so building Doux needs no Faust toolchain at all. A normal `cargo build` simply compiles the generated Rust, and the only Faust-related dependency it pulls is `faust-types`, a small pure-Rust crate that provides the runtime traits the generated code implements.

The Faust compiler is needed only to regenerate that code, which concerns contributors who change the DSP. After editing a `.dsp` source, run `dsp/regen.sh` to rebuild the generated modules, then commit the result; the generated files are never edited by hand.

```bash
dsp/regen.sh           # regenerate src/effects/faust_dsp/*_gen.rs from dsp/*.dsp
dsp/regen.sh --check   # verify the committed code is in sync with the sources
```

The script expects the pinned `faust` version (see `dsp/regen.sh`) on your `PATH`, so its output stays reproducible.

### WASM

To run Doux in the browser, build the WebAssembly module.

```bash
./build-wasm.sh          # → website/static/doux.wasm
```

This compiles the `wasm32-unknown-unknown` target with `--no-default-features`, producing a module that is driven from a browser AudioWorklet (see `src/wasm.rs`).

### Platform notes

On macOS the CoreAudio backend works without any extra setup. On Windows, Doux uses WASAPI by default and can use ASIO when it is built with `--features asio`. On Linux there is a little more to know, which the next section covers.

## CLI reference

### doux — OSC server

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--samples` | `-s` | Directory of audio samples | — |
| `--port` | `-p` | OSC port (UDP) | 57120 |
| `--list-devices` | | List audio devices and exit | — |
| `--input` | `-i` | Input device (name or index) | — |
| `--output` | `-o` | Output device (name or index) | — |
| `--channels` | | Output channels | 2 |
| `--buffer-size` | `-b` | Device buffer in samples; ignored when the host fixes it | host default |
| `--dsp-block-size` | | Inner DSP block in samples (1–256) | 32 |
| `--max-voices` | | Polyphony cap | 32 |
| `--preload` | | Decode all samples at startup | false |
| `--host` | | Audio host backend (platform-specific; see [Linux audio](#linux-audio)) | auto |
| `--diagnose` | | Print audio diagnostics and exit | — |

### doux-repl — interactive REPL

`doux-repl` is a small interpreter for testing and quick experiments. Type an event to play it, or one of the `.` commands below.

```text
.hush     fade out all voices       .voices   active voice count
.panic    silence immediately       .stats    engine telemetry
.reset    reset engine state        .help     list commands
                                     .quit     exit
```

It accepts the same audio flags as `doux`, apart from `--port` and `--preload`.

### doux-render — offline renderer

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--duration` | `-d` | Seconds to render | required |
| `--output` | `-o` | Output WAV path | required |
| `--eval` | `-e` | Pattern to evaluate (repeatable) | — |
| `--samples` | `-s` | Directory of audio samples | — |
| `--sample-rate` | | Sample rate (Hz) | 48000 |
| `--channels` | | Output channels | 2 |
| `--max-voices` | | Polyphony cap | 64 |
| `--dsp-block-size` | | Inner DSP block in samples (1–256) | 32 |

## Linux audio

On Linux, Doux speaks to PipeWire natively, so there is no need to wrap it with `pw-jack`. JACK, PulseAudio, and ALSA are compiled in as well. Under the default `--host auto`, Doux selects the first backend that is actually available, trying pipewire, then jack, then pulseaudio, then alsa. Pass `--host pipewire|jack|pulseaudio|alsa` to force a particular one. The PulseAudio backend is written in pure Rust and needs no system packages, and since PipeWire ships a PulseAudio-compatible layer (pipewire-pulse), that backend covers PipeWire systems too.

Building the native PipeWire backend needs its development headers and libclang, so install those before you build.

```bash
# Debian/Ubuntu
sudo apt install pkg-config libclang-dev libpipewire-0.3-dev libasound2-dev libjack-jackd2-dev

# Fedora
sudo dnf install pkgconf-pkg-config clang pipewire-devel alsa-lib-devel jack-audio-connection-kit-devel

# Arch
sudo pacman -S pkgconf clang pipewire alsa-lib jack2
```

To reach ALSA hardware directly, add your user to the `audio` group and then start a fresh login session.

```bash
sudo usermod -aG audio $USER
```

When something is wrong, `doux --diagnose` is the first thing to run: it reports the available hosts, the JACK (`jack_lsp`) and PipeWire (`pw-cli`) server status, and whether the default device is reachable. If it finds no devices, confirm that you are in the `audio` group with `id -Gn`. If it selects the wrong device, list the options with `--list-devices` and choose one explicitly with `--output`.

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) to get set up. The full release history lives in [CHANGELOG.md](CHANGELOG.md).

## License

Doux is licensed under the [GNU Affero General Public License v3.0](LICENSE).
