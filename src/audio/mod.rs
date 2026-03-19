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

/// Gets an audio host by selection mode.
pub fn get_host(selection: HostSelection) -> Result<Host, DouxError> {
    match selection {
        HostSelection::Auto => Ok(preferred_host()),
        HostSelection::Named(name) => {
            for host_id in cpal::available_hosts() {
                if host_id.name().to_lowercase().contains(&name) {
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
const PREFERRED_HOSTS: &[&str] = &["pipewire", "jack"];

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
    if let Ok(idx) = spec.parse::<usize>() {
        return devices.into_iter().nth(idx);
    }
    let spec_lower = spec.to_lowercase();
    devices.into_iter().find(|d| {
        d.description()
            .map(|desc| desc.name().to_lowercase().contains(&spec_lower))
            .unwrap_or(false)
    })
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

/// Returns true if the given host controls its own buffer size.
/// JACK and ASIO enforce their own buffer sizes, so user-specified values should be ignored.
pub fn host_controls_buffer_size(host: &Host) -> bool {
    let name = host.id().name().to_lowercase();
    name.contains("jack") || name.contains("asio")
}

/// Gets the maximum number of output channels supported by a device.
pub fn max_output_channels(device: &Device) -> u16 {
    device
        .supported_output_configs()
        .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(2))
        .unwrap_or(2)
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
