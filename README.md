<h1 align="center">Doux</h1>

<p align="center"><em>A Software Synthesizer Engine for Live Coding</em></p>

Written in Rust, initially ported from [Dough](https://dough.strudel.cc/) by Felix Roos and co. Online documentation and live playground are accessible through the demo website: [doux](https://doux.livecoding.fr). Doux uses a fixed architecture and provides various oscillators, filters and effects. It is both capable of synthesis and sampling. Doux is made to be integrated in other audio applications, both through its native or WASM version.

## CLI Flags

### doux (OSC server)

Doux is the engine itself, that you can test as a standalone binary.

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--samples` | `-s` | Directory containing audio samples | - |
| `--port` | `-p` | OSC port to listen on | 57120 |
| `--list-devices` | | List available audio devices and exit | - |
| `--input` | `-i` | Input device (name or index) | - |
| `--output` | `-o` | Output device (name or index) | - |
| `--channels` | | Number of output channels | 2 |
| `--buffer-size` | `-b` | Audio buffer size in samples | system |
| `--max-voices` | | Maximum polyphony | 32 |
| `--preload` | | Preload all samples at startup | false |
| `--host` | | Audio host: pipewire, pulseaudio, jack, alsa, asio, auto | auto |
| `--diagnose` | | Run audio diagnostics and exit | - |

### doux-repl (interactive REPL)

Doux-REPL is a small interpreter mostly used for debugging and testing.

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--samples` | `-s` | Directory containing audio samples | - |
| `--list-devices` | | List available audio devices and exit | - |
| `--input` | `-i` | Input device (name or index) | - |
| `--output` | `-o` | Output device (name or index) | - |
| `--channels` | | Number of output channels | 2 |
| `--buffer-size` | `-b` | Audio buffer size in samples | system |
| `--max-voices` | | Maximum polyphony | 32 |
| `--host` | | Audio host: pipewire, pulseaudio, jack, alsa, asio, auto | auto |
| `--diagnose` | | Run audio diagnostics and exit | - |

### doux-render (offline rendering)

Doux-render renders audio synthesis to a WAV file instead of real-time playback.

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--duration` | `-d` | Duration to render in seconds | required |
| `--eval` | `-e` | Command to evaluate (can be repeated) | - |
| `--output` | `-o` | Output WAV file path | required |
| `--samples` | `-s` | Directory containing audio samples | - |
| `--sample-rate` | | Sample rate in Hz | 48000 |
| `--channels` | | Number of output channels | 2 |
| `--max-voices` | | Maximum polyphony | 64 |

### Performance workflow

Use the native benchmark tool for repeatable engine measurements:

```bash
cargo run --release --bin doux-bench -- suite
cargo run --release --features profiling --bin doux-bench -- case voice_stress --breakdown
cargo bench
```

`doux-bench` runs a checked-in workload corpus and reports wall time, realtime factor,
`ns/sample`, and `ns/block`. With the `profiling` feature enabled it also prints
aggregate engine phase timings for schedule processing, sample upgrades, voice
source generation, voice FX, orbit FX, final mix, recorder capture, and total block time.

Supported sampled-profiler workflows:

- macOS: run `doux-bench case <name>` under Instruments Time Profiler
- Linux: run `perf record --call-graph dwarf ./target/release/doux-bench case <name>` and inspect the capture with your flamegraph workflow

## Linux Audio Setup

On Linux, doux talks to PipeWire natively (default on most modern distributions) — no `pw-jack` wrapper needed. PulseAudio, JACK, and ALSA backends are also compiled in. With `--host auto` (the default), the priority is pipewire > jack > pulseaudio > alsa.

### Quick Start

```bash
# Just run it — PipeWire is picked up automatically
doux

# Or run diagnostics to check your audio setup
doux --diagnose
```

Force a specific backend with `--host pipewire|pulseaudio|jack|alsa`. The PulseAudio backend is pure Rust and needs no extra packages; it also covers PipeWire systems via pipewire-pulse.

### Building from Source

The native PipeWire backend requires PipeWire ≥ 0.3.53 dev headers and libclang at build time:

```bash
# Debian/Ubuntu
sudo apt install pkg-config libclang-dev libpipewire-0.3-dev libasound2-dev libjack-jackd2-dev

# Fedora
sudo dnf install pkgconf-pkg-config clang pipewire-devel alsa-lib-devel jack-audio-connection-kit-devel

# Arch
sudo pacman -S pkgconf clang pipewire alsa-lib jack2
```

For direct ALSA access, ensure your user is in the audio group:

```bash
sudo usermod -aG audio $USER
# Log out and back in for group change to take effect
```

### Troubleshooting

Run `doux --diagnose` to check:
- Available audio hosts
- User group membership (audio, pipewire)
- JACK/PipeWire server status
- Default device accessibility

Common issues:
- **No devices found**: Check group membership (`id -Gn` should show `audio`)
- **Wrong device selected**: Use `--list-devices` and specify with `--output`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).
