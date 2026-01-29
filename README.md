<h1 align="center">doux</h1>

<p align="center"><em>A Software Synthesizer Engine for Live Coding</em></p>

Written in Rust, initially ported from [Dough](https://dough.strudel.cc/) by Felix Roos and al.
Documentation and live playground can be found on this page: [doux](https://doux.livecoding.fr).

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
