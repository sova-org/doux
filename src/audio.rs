//! Audio device enumeration and stream creation utilities.
//!
//! Provides functions to list available audio devices and create audio streams
//! with specific configurations.

use crate::error::DouxError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SupportedStreamConfig};

/// Information about an available audio device.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub index: usize,
    pub max_channels: u16,
    pub is_default: bool,
}

/// Information about an available audio host.
#[derive(Debug, Clone)]
pub struct AudioHostInfo {
    pub name: String,
    pub available: bool,
}

/// Host selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HostSelection {
    /// Try JACK first, fallback to ALSA (default behavior).
    #[default]
    Auto,
    /// Use JACK backend explicitly.
    Jack,
    /// Use ALSA backend explicitly.
    Alsa,
}

impl std::str::FromStr for HostSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(HostSelection::Auto),
            "jack" => Ok(HostSelection::Jack),
            "alsa" => Ok(HostSelection::Alsa),
            _ => Err(format!("unknown host: {s} (use: auto, jack, alsa)")),
        }
    }
}

impl std::fmt::Display for HostSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostSelection::Auto => write!(f, "auto"),
            HostSelection::Jack => write!(f, "jack"),
            HostSelection::Alsa => write!(f, "alsa"),
        }
    }
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

/// Gets an audio host by selection mode.
pub fn get_host(selection: HostSelection) -> Result<Host, DouxError> {
    match selection {
        HostSelection::Auto => Ok(preferred_host()),
        HostSelection::Jack => {
            for host_id in cpal::available_hosts() {
                if host_id.name().to_lowercase().contains("jack") {
                    if let Ok(host) = cpal::host_from_id(host_id) {
                        return Ok(host);
                    }
                }
            }
            Err(DouxError::HostNotFound("jack".to_string()))
        }
        HostSelection::Alsa => {
            for host_id in cpal::available_hosts() {
                if host_id.name().to_lowercase().contains("alsa") {
                    if let Ok(host) = cpal::host_from_id(host_id) {
                        return Ok(host);
                    }
                }
            }
            Err(DouxError::HostNotFound("alsa".to_string()))
        }
    }
}

/// Returns the preferred audio host, trying JACK first (works with pipewire-jack on Linux).
pub fn preferred_host() -> Host {
    for host_id in cpal::available_hosts() {
        if host_id.name().to_lowercase().contains("jack") {
            if let Ok(host) = cpal::host_from_id(host_id) {
                return host;
            }
        }
    }
    cpal::default_host()
}

/// Returns the default CPAL host for the current platform.
pub fn default_host() -> Host {
    preferred_host()
}

/// Lists all available output audio devices.
pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
    let host = default_host();
    let default_name = host
        .default_output_device()
        .and_then(|d| d.name().ok());

    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };

    devices
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device.name().ok()?;
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

/// Lists all available input audio devices.
pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
    let host = default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let Ok(devices) = host.input_devices() else {
        return Vec::new();
    };

    devices
        .enumerate()
        .filter_map(|(index, device)| {
            let name = device.name().ok()?;
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

/// Finds an output device by index or partial name match.
///
/// If `spec` parses as a number, returns the device at that index.
/// Otherwise, performs a case-insensitive substring match on device names.
pub fn find_output_device(spec: &str) -> Option<Device> {
    let host = default_host();
    let devices = host.output_devices().ok()?;
    find_device_impl(devices, spec)
}

/// Finds an input device by index or partial name match.
pub fn find_input_device(spec: &str) -> Option<Device> {
    let host = default_host();
    let devices = host.input_devices().ok()?;
    find_device_impl(devices, spec)
}

fn find_device_impl<I>(devices: I, spec: &str) -> Option<Device>
where
    I: Iterator<Item = Device>,
{
    let devices: Vec<_> = devices.collect();
    if let Ok(idx) = spec.parse::<usize>() {
        return devices.into_iter().nth(idx);
    }
    let spec_lower = spec.to_lowercase();
    devices.into_iter().find(|d| {
        d.name()
            .map(|n| n.to_lowercase().contains(&spec_lower))
            .unwrap_or(false)
    })
}

/// Returns the default output device.
pub fn default_output_device() -> Option<Device> {
    default_host().default_output_device()
}

/// Returns the default input device.
pub fn default_input_device() -> Option<Device> {
    default_host().default_input_device()
}

/// Gets the default output config for a device.
pub fn default_output_config(device: &Device) -> Option<SupportedStreamConfig> {
    device.default_output_config().ok()
}

/// Gets the maximum number of output channels supported by a device.
pub fn max_output_channels(device: &Device) -> u16 {
    device
        .supported_output_configs()
        .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(2))
        .unwrap_or(2)
}

/// Diagnostic result for Linux audio setup.
#[derive(Debug)]
pub struct DiagnosticResult {
    pub label: String,
    pub status: DiagnosticStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStatus {
    Ok,
    Warn,
    Error,
}

impl DiagnosticResult {
    fn ok(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Ok,
            message: message.to_string(),
        }
    }

    fn warn(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Warn,
            message: message.to_string(),
        }
    }

    fn error(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Error,
            message: message.to_string(),
        }
    }
}

/// Runs audio diagnostics (primarily useful on Linux).
pub fn run_diagnostics() -> Vec<DiagnosticResult> {
    let mut results = Vec::new();

    // Check available hosts
    let hosts = list_hosts();
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

    // Check default host and devices
    let host = preferred_host();
    let host_name = host.id().name();
    results.push(DiagnosticResult::ok("Active host", host_name));

    match host.default_output_device() {
        Some(device) => {
            let name = device.name().unwrap_or_else(|_| "unknown".to_string());
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
            let name = device.name().unwrap_or_else(|_| "unknown".to_string());
            results.push(DiagnosticResult::ok("Default input", &name));
        }
        None => {
            results.push(DiagnosticResult::warn(
                "Default input",
                "no default input device",
            ));
        }
    }

    // Linux-specific checks
    #[cfg(target_os = "linux")]
    {
        results.extend(run_linux_diagnostics());
    }

    results
}

#[cfg(target_os = "linux")]
fn run_linux_diagnostics() -> Vec<DiagnosticResult> {
    let mut results = Vec::new();

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
