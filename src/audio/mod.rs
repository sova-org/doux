//! Audio device enumeration and stream creation utilities.
//!
//! Provides functions to list available audio devices and create audio streams
//! with specific configurations.

mod common;

pub use common::{AudioDeviceInfo, AudioHostInfo, DiagnosticResult, DiagnosticStatus};
pub use cpal;

use crate::error::DouxError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SupportedStreamConfig};

/// Host selection mode — OS-agnostic, CPAL resolves backend availability at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HostSelection {
    #[default]
    Auto,
    Named(String),
}

impl std::str::FromStr for HostSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(HostSelection::Auto),
            other => Ok(HostSelection::Named(other.to_string())),
        }
    }
}

impl std::fmt::Display for HostSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostSelection::Auto => write!(f, "auto"),
            HostSelection::Named(s) => write!(f, "{s}"),
        }
    }
}

/// Whether the session manager wires our stream ports to the system devices, or
/// leaves them for the user to patch. PipeWire and JACK only; every other
/// backend connects and has no say in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Patching {
    #[default]
    Auto,
    Manual,
}

/// True when this build can register the host's ports and leave them unlinked.
/// Answered by trying to construct the unwired host, not by matching a backend
/// name: `unwired_host` is Linux-only, so a JACK host on macOS or Windows names
/// a backend that has the feature in cpal but no route to it from here.
pub fn host_supports_manual_patching(host: &Host) -> bool {
    unwired_host(host.id().name()).is_some()
}

/// Rebuilds `name`'s host with auto-connection off. `None` when the backend has
/// no such switch, which leaves the caller with the connecting host it had.
#[cfg(target_os = "linux")]
fn unwired_host(name: &str) -> Option<Host> {
    let name = name.to_lowercase();
    if name.contains("pipewire") {
        use cpal::platform::PipeWireHost;
        let mut host = PipeWireHost::new().ok()?;
        host.set_connect_automatically(false);
        return Some(host.into());
    }
    if name.contains("jack") {
        use cpal::platform::JackHost;
        let mut host = JackHost::new().ok()?;
        host.set_connect_automatically(false);
        return Some(host.into());
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn unwired_host(_name: &str) -> Option<Host> {
    None
}

/// Gets an audio host by selection mode, wired up by the session manager.
pub fn get_host(selection: HostSelection) -> Result<Host, DouxError> {
    get_host_patched(selection, Patching::Auto)
}

/// Gets an audio host, choosing whether its ports come pre-wired.
pub fn get_host_patched(selection: HostSelection, patching: Patching) -> Result<Host, DouxError> {
    let host = get_host_wired(selection)?;
    if patching == Patching::Auto {
        return Ok(host);
    }
    // Falls back to the wired host rather than failing: a backend without the
    // switch still has to make sound.
    Ok(unwired_host(host.id().name()).unwrap_or(host))
}

fn get_host_wired(selection: HostSelection) -> Result<Host, DouxError> {
    match selection {
        HostSelection::Auto => Ok(preferred_host()),
        HostSelection::Named(name) => {
            for host_id in cpal::available_hosts() {
                if host_id.name().to_lowercase().contains(&name.to_lowercase()) {
                    if let Ok(host) = cpal::host_from_id(host_id) {
                        return Ok(host);
                    }
                }
            }
            Err(DouxError::HostNotFound(name))
        }
    }
}

#[cfg(target_os = "linux")]
const PREFERRED_HOSTS: &[&str] = &["pipewire", "jack", "pulseaudio"];

#[cfg(all(target_os = "windows", feature = "asio"))]
const PREFERRED_HOSTS: &[&str] = &["asio", "wasapi"];

#[cfg(all(target_os = "windows", not(feature = "asio")))]
const PREFERRED_HOSTS: &[&str] = &["wasapi"];

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
const PREFERRED_HOSTS: &[&str] = &[];

/// Returns the preferred audio host for the current platform.
/// Tries platform-preferred backends in order, falling back to CPAL's default.
pub fn preferred_host() -> Host {
    let hosts = cpal::available_hosts();
    for preferred in PREFERRED_HOSTS {
        for &host_id in &hosts {
            if host_id.name().to_lowercase().contains(preferred) {
                if let Ok(host) = cpal::host_from_id(host_id) {
                    if host.default_output_device().is_some() {
                        return host;
                    }
                }
            }
        }
    }
    cpal::default_host()
}

/// Lists available audio hosts on the system.
pub fn list_hosts() -> Vec<AudioHostInfo> {
    cpal::available_hosts()
        .into_iter()
        .map(|id| AudioHostInfo {
            name: id.name().to_string(),
            available: cpal::host_from_id(id).is_ok(),
        })
        .collect()
}

/// Lists all available output audio devices for a given host.
pub fn list_output_devices_for(host: &Host) -> Vec<AudioDeviceInfo> {
    let default_name = host
        .default_output_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };

    devices
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "<Unknown>".to_string());
            let max_channels = device
                .supported_output_configs()
                .ok()?
                .map(|c| c.channels())
                .max()
                .unwrap_or(2);
            let is_default = Some(&name) == default_name.as_ref();
            Some(AudioDeviceInfo {
                name,
                index,
                max_channels,
                is_default,
            })
        })
        .collect()
}

/// Lists all available output audio devices using the preferred host.
pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
    list_output_devices_for(&preferred_host())
}

/// Lists all available input audio devices for a given host.
pub fn list_input_devices_for(host: &Host) -> Vec<AudioDeviceInfo> {
    let default_name = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    let Ok(devices) = host.input_devices() else {
        return Vec::new();
    };

    devices
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "<Unknown>".to_string());
            let max_channels = device
                .supported_input_configs()
                .ok()?
                .map(|c| c.channels())
                .max()
                .unwrap_or(2);
            let is_default = Some(&name) == default_name.as_ref();
            Some(AudioDeviceInfo {
                name,
                index,
                max_channels,
                is_default,
            })
        })
        .collect()
}

/// Lists all available input audio devices using the preferred host.
pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
    list_input_devices_for(&preferred_host())
}

/// Finds an output device by index or partial name match using a given host.
pub fn find_output_device_for(host: &Host, spec: &str) -> Option<Device> {
    let devices = host.output_devices().ok()?;
    find_device(devices, spec)
}

/// Finds an output device by index or partial name match using the preferred host.
pub fn find_output_device(spec: &str) -> Option<Device> {
    find_output_device_for(&preferred_host(), spec)
}

/// Finds an input device by index or partial name match using a given host.
pub fn find_input_device_for(host: &Host, spec: &str) -> Option<Device> {
    let devices = host.input_devices().ok()?;
    find_device(devices, spec)
}

/// Finds an input device by index or partial name match using the preferred host.
pub fn find_input_device(spec: &str) -> Option<Device> {
    find_input_device_for(&preferred_host(), spec)
}

pub fn find_device<I>(devices: I, spec: &str) -> Option<Device>
where
    I: Iterator<Item = Device>,
{
    let devices: Vec<_> = devices.collect();

    let names: Vec<Option<String>> = devices
        .iter()
        .map(|d| d.description().ok().map(|desc| desc.name().to_string()))
        .collect();

    let spec_lower = spec.to_lowercase();

    let pos = names
        .iter()
        .position(|n| n.as_deref() == Some(spec))
        .or_else(|| {
            names.iter().position(|n| {
                n.as_ref()
                    .map(|n| n.to_lowercase() == spec_lower)
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            names.iter().position(|n| {
                n.as_ref()
                    .map(|n| n.to_lowercase().contains(&spec_lower))
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            spec.parse::<usize>()
                .ok()
                .filter(|&idx| idx < devices.len())
        });

    pos.map(|i| devices.into_iter().nth(i).unwrap())
}

/// Returns the default output device for a given host.
pub fn default_output_device_for(host: &Host) -> Option<Device> {
    host.default_output_device()
}

/// Returns the default output device.
/// On Linux with JACK, uses a named client ("doux").
pub fn default_output_device() -> Option<Device> {
    let host = preferred_host();

    #[cfg(target_os = "linux")]
    if host.id().name().to_lowercase().contains("jack") {
        if let Some(device) = jack_output_device("doux") {
            return Some(device);
        }
    }

    host.default_output_device()
}

/// Returns the default input device for a given host.
pub fn default_input_device_for(host: &Host) -> Option<Device> {
    host.default_input_device()
}

/// Returns the default input device.
/// On Linux with JACK, uses a named client ("doux").
pub fn default_input_device() -> Option<Device> {
    let host = preferred_host();

    #[cfg(target_os = "linux")]
    if host.id().name().to_lowercase().contains("jack") {
        if let Some(device) = jack_input_device("doux") {
            return Some(device);
        }
    }

    host.default_input_device()
}

#[cfg(target_os = "linux")]
fn jack_output_device(client_name: &str) -> Option<Device> {
    use cpal::platform::JackHost;
    let mut host = JackHost::new().ok()?;
    let jack_dev = host.output_device_with_name(client_name)?;
    Some(jack_dev.into())
}

#[cfg(target_os = "linux")]
fn jack_input_device(client_name: &str) -> Option<Device> {
    use cpal::platform::JackHost;
    let mut host = JackHost::new().ok()?;
    let jack_dev = host.input_device_with_name(client_name)?;
    Some(jack_dev.into())
}

/// Gets the default output config for a device.
pub fn default_output_config(device: &Device) -> Option<SupportedStreamConfig> {
    device.default_output_config().ok()
}

/// Gets the default input config for a device.
pub fn default_input_config(device: &Device) -> Option<SupportedStreamConfig> {
    device.default_input_config().ok()
}

/// Returns true if the given host controls its own buffer size.
/// JACK and ASIO enforce their own buffer sizes, so user-specified values should be ignored.
pub fn host_controls_buffer_size(host: &Host) -> bool {
    let name = host.id().name().to_lowercase();
    name.contains("jack") || name.contains("asio")
}

/// Usable output channel count at `sample_rate`: honor `requested`.
/// PipeWire/JACK accept counts that `supported_output_configs()` under-reports as
/// stereo, so probe by actually opening a stream; on real refusal (hardware limit),
/// warn and fall back to the device default.
///
/// Takes the rate rather than assuming the device's own, because a device can
/// support 8 channels and 96 kHz separately but not together. Resolve the rate
/// first and pass it here, so what gets probed is the combination that will
/// actually be opened.
pub fn resolve_output_channels(device: &Device, requested: u16, sample_rate: u32) -> u16 {
    let requested = requested.max(1); // PipeWire 0.18 rejects channels == 0
    let Some(default_cfg) = default_output_config(device) else {
        return requested; // cannot probe; trust the request
    };
    match probe_output(device, requested, sample_rate, &default_cfg) {
        Ok(()) => requested,
        Err(e) => {
            let fallback = default_cfg.channels();
            eprintln!("[doux] {requested} output channels refused ({e}); using {fallback}");
            fallback
        }
    }
}

/// Opens and drops a playback stream to find out whether the device takes this
/// shape. Raw builder, so the probe runs at the device's own sample format
/// rather than assuming `f32`.
fn probe_output(
    device: &Device,
    channels: u16,
    sample_rate: u32,
    default_cfg: &SupportedStreamConfig,
) -> Result<(), cpal::Error> {
    let stream = device.build_output_stream_raw(
        cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        },
        default_cfg.sample_format(),
        |_: &mut cpal::Data, _: &cpal::OutputCallbackInfo| {},
        |_err: cpal::Error| {},
        None,
    )?;
    drop(stream); // accepted at build time; no play() needed
    Ok(())
}

/// Usable output sample rate: honor `requested`, `None` meaning the device's own.
/// Probed like the channel counts, since a backend that resamples will accept a
/// rate its default config never mentions.
pub fn resolve_sample_rate(device: &Device, requested: Option<u32>) -> u32 {
    let Some(default_cfg) = default_output_config(device) else {
        return requested.unwrap_or(44_100);
    };
    let Some(rate) = requested.filter(|&r| r > 0) else {
        return default_cfg.sample_rate();
    };
    if rate == default_cfg.sample_rate() {
        return rate;
    }
    match probe_output(device, default_cfg.channels(), rate, &default_cfg) {
        Ok(()) => rate,
        Err(e) => {
            let fallback = default_cfg.sample_rate();
            eprintln!("[doux] {rate} Hz refused ({e}); using {fallback}");
            fallback
        }
    }
}

/// The output shape a device will actually take: channel count and sample rate
/// resolved together, because they are accepted together. A device can support 8
/// channels and 96 kHz on their own and refuse the pair, so resolving them
/// independently and combining the answers yields a config that never opens, and
/// `build_stream` fails outright instead of degrading.
///
/// Falls all the way back to the device's own default config if even the settled
/// combination is refused, so the caller always gets something openable.
pub fn resolve_output_shape(
    device: &Device,
    requested_channels: u16,
    requested_rate: Option<u32>,
) -> (u16, u32) {
    let rate = resolve_sample_rate(device, requested_rate);
    let channels = resolve_output_channels(device, requested_channels, rate);
    let Some(default_cfg) = default_output_config(device) else {
        return (channels, rate);
    };
    if (channels, rate) == (default_cfg.channels(), default_cfg.sample_rate()) {
        return (channels, rate);
    }
    match probe_output(device, channels, rate, &default_cfg) {
        Ok(()) => (channels, rate),
        Err(e) => {
            let fallback = (default_cfg.channels(), default_cfg.sample_rate());
            eprintln!(
                "[doux] {channels}ch at {rate} Hz refused together ({e}); using {}ch at {} Hz",
                fallback.0, fallback.1
            );
            fallback
        }
    }
}

/// Usable input channel count: honor `requested`.
/// `default_input_config()` reports stereo for a 4-in interface on every backend
/// (cpal's default heuristic ranks 2 channels highest, and the PipeWire host's
/// default-input node is hardcoded to 2), so probe by actually opening a stream
/// rather than trusting what the device volunteers.
pub fn resolve_input_channels(device: &Device, requested: u16) -> u16 {
    let requested = requested.max(1); // PipeWire 0.18 rejects channels == 0
    let Some(default_cfg) = default_input_config(device) else {
        return requested; // cannot probe; trust the request
    };
    match probe_input(device, requested, default_cfg.sample_rate(), &default_cfg) {
        Ok(()) => requested,
        Err(e) => {
            let fallback = default_cfg.channels();
            eprintln!("[doux] {requested} input channels refused ({e}); using {fallback}");
            fallback
        }
    }
}

/// Usable input sample rate: honor `requested`, which the caller sets to the
/// output's rate so a duplex device runs both halves off one clock. Nothing
/// resamples the live input, so a split here is drift.
pub fn resolve_input_rate(device: &Device, requested: u32) -> u32 {
    let Some(default_cfg) = default_input_config(device) else {
        return requested;
    };
    if requested == default_cfg.sample_rate() || requested == 0 {
        return default_cfg.sample_rate();
    }
    match probe_input(device, default_cfg.channels(), requested, &default_cfg) {
        Ok(()) => requested,
        Err(_) => default_cfg.sample_rate(),
    }
}

/// Opens and drops a capture stream to find out whether the device takes this
/// shape. Uses the raw builder so the probe runs at the device's own sample
/// format: a hardcoded `f32` would be refused outright on an I16-native
/// interface and make every count look unsupported.
fn probe_input(
    device: &Device,
    channels: u16,
    sample_rate: u32,
    default_cfg: &SupportedStreamConfig,
) -> Result<(), cpal::Error> {
    let stream = device.build_input_stream_raw(
        cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        },
        default_cfg.sample_format(),
        |_: &cpal::Data, _: &cpal::InputCallbackInfo| {},
        |_err: cpal::Error| {},
        None,
    )?;
    drop(stream); // accepted at build time; no play() needed
    Ok(())
}

/// Runs audio diagnostics.
pub fn run_diagnostics() -> Vec<DiagnosticResult> {
    let hosts = list_hosts();
    let mut results = Vec::new();

    for host in &hosts {
        if host.available {
            results.push(DiagnosticResult::ok(
                "Host",
                &format!("{} available", host.name),
            ));
        } else {
            results.push(DiagnosticResult::warn(
                "Host",
                &format!("{} not available", host.name),
            ));
        }
    }

    let host = preferred_host();
    let host_name = host.id().name();

    #[cfg(target_os = "linux")]
    {
        let reason = if hosts
            .iter()
            .any(|h| h.name.to_lowercase().contains("pipewire") && h.available)
        {
            "pipewire preferred"
        } else if hosts
            .iter()
            .any(|h| h.name.to_lowercase().contains("jack") && h.available)
        {
            "jack preferred"
        } else if hosts
            .iter()
            .any(|h| h.name.to_lowercase().contains("pulseaudio") && h.available)
        {
            "pulseaudio preferred"
        } else {
            "fallback"
        };
        results.push(DiagnosticResult::ok(
            "Active host",
            &format!("{host_name} ({reason})"),
        ));
    }

    #[cfg(not(target_os = "linux"))]
    results.push(DiagnosticResult::ok("Active host", host_name));

    match host.default_output_device() {
        Some(device) => {
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            results.push(DiagnosticResult::ok("Default output", &name));
        }
        None => {
            results.push(DiagnosticResult::error(
                "Default output",
                "no default output device",
            ));
        }
    }

    match host.default_input_device() {
        Some(device) => {
            let name = device
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            results.push(DiagnosticResult::ok("Default input", &name));
        }
        None => {
            results.push(DiagnosticResult::warn(
                "Default input",
                "no default input device",
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("jack_lsp").output() {
            if output.status.success() {
                results.push(DiagnosticResult::ok("JACK", "server reachable"));
            }
        }

        if let Ok(output) = std::process::Command::new("pw-cli").arg("info").output() {
            if output.status.success() {
                results.push(DiagnosticResult::ok("PipeWire", "running"));
            }
        }

        if std::path::Path::new("/usr/lib/alsa-lib/libasound_module_pcm_pipewire.so").exists()
            || std::path::Path::new("/usr/lib64/alsa-lib/libasound_module_pcm_pipewire.so").exists()
        {
            results.push(DiagnosticResult::ok(
                "pipewire-alsa",
                "installed (MIDI bridge available)",
            ));
        } else if hosts
            .iter()
            .any(|h| h.name.to_lowercase().contains("pipewire"))
        {
            results.push(DiagnosticResult::warn(
                "pipewire-alsa",
                "not found — MIDI ports may not be visible (install pipewire-alsa)",
            ));
        }
    }

    results
}

/// Prints diagnostic results to stdout.
pub fn print_diagnostics() {
    let results = run_diagnostics();
    for r in results {
        let prefix = match r.status {
            DiagnosticStatus::Ok => "\x1b[32m[OK]\x1b[0m",
            DiagnosticStatus::Warn => "\x1b[33m[WARN]\x1b[0m",
            DiagnosticStatus::Error => "\x1b[31m[ERROR]\x1b[0m",
        };
        println!("{} {}: {}", prefix, r.label, r.message);
    }
}
