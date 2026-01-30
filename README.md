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
| `--host` | | Audio host: jack, alsa, auto | auto |
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
| `--host` | | Audio host: jack, alsa, auto | auto |
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

## Linux Audio Setup

On Linux, doux supports both JACK and ALSA backends. For systems using PipeWire (default on most modern distributions), the JACK backend via `pipewire-jack` provides the best compatibility.

### Quick Start

```bash
# Use JACK backend (recommended for PipeWire systems)
doux --host jack

# Or run diagnostics to check your audio setup
doux --diagnose
```

### ALSA Fallback

For direct ALSA access without PipeWire:

```bash
# Install ALSA dev libraries
sudo apt install libasound2-dev  # Debian/Ubuntu
sudo dnf install alsa-lib-devel  # Fedora

# Ensure user is in audio group
sudo usermod -aG audio $USER
# Log out and back in for group change to take effect
```

Then run with `--host alsa`.

### Troubleshooting

Run `doux --diagnose` to check:
- Available audio hosts
- User group membership (audio, pipewire)
- JACK server status
- Default device accessibility

Common issues:
- **No devices found**: Check group membership (`id -Gn` should show `audio`)
- **JACK unavailable**: Start JACK server or install pipewire-jack
- **Wrong device selected**: Use `--list-devices` and specify with `--output`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).
