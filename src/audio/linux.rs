//! Linux-specific audio implementation (JACK/ALSA).

use super::common::{AudioHostInfo, DiagnosticResult};
use crate::error::DouxError;
use cpal::platform::{DeviceInner, JackDevice};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

/// Host selection mode for Linux.
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

/// Returns the preferred audio host, trying JACK first (works with pipewire-jack).
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

/// Creates a JACK output device with a custom client name.
fn jack_output_device(client_name: &str, connect_automatically: bool) -> Option<Device> {
    let jack_dev =
        JackDevice::default_output_device(client_name, connect_automatically, false).ok()?;
    Some(Device::from(DeviceInner::Jack(jack_dev)))
}

/// Creates a JACK input device with a custom client name.
fn jack_input_device(client_name: &str, connect_automatically: bool) -> Option<Device> {
    let jack_dev =
        JackDevice::default_input_device(client_name, connect_automatically, false).ok()?;
    Some(Device::from(DeviceInner::Jack(jack_dev)))
}

/// Returns the default output device.
/// Uses JACK with "doux" as the client name if available.
pub fn default_output_device() -> Option<Device> {
    if let Some(device) = jack_output_device("doux", true) {
        return Some(device);
    }
    preferred_host().default_output_device()
}

/// Returns the default input device.
/// Uses JACK with "doux" as the client name if available.
pub fn default_input_device() -> Option<Device> {
    if let Some(device) = jack_input_device("doux", true) {
        return Some(device);
    }
    preferred_host().default_input_device()
}

/// Runs Linux-specific audio diagnostics.
pub fn run_diagnostics(hosts: &[AudioHostInfo]) -> Vec<DiagnosticResult> {
    let mut results = Vec::new();

    for host in hosts {
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

    // JACK server check
    if let Ok(output) = std::process::Command::new("jack_lsp").output() {
        if output.status.success() {
            results.push(DiagnosticResult::ok("JACK", "server reachable"));
        }
    }

    // PipeWire check
    if let Ok(output) = std::process::Command::new("pw-cli").arg("info").output() {
        if output.status.success() {
            results.push(DiagnosticResult::ok("PipeWire", "running"));
        }
    }

    results
}
