//! Windows-specific audio implementation (WASAPI).

use super::common::{AudioHostInfo, DiagnosticResult};
use crate::error::DouxError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

/// Host selection mode for Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HostSelection {
    #[default]
    Auto,
    /// Use WASAPI backend explicitly.
    Wasapi,
}

impl std::str::FromStr for HostSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(HostSelection::Auto),
            "wasapi" => Ok(HostSelection::Wasapi),
            "jack" | "alsa" => Err(format!(
                "{s} is not available on Windows (use: auto, wasapi)"
            )),
            _ => Err(format!("unknown host: {s} (use: auto, wasapi)")),
        }
    }
}

impl std::fmt::Display for HostSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostSelection::Auto => write!(f, "auto"),
            HostSelection::Wasapi => write!(f, "wasapi"),
        }
    }
}

/// Gets an audio host by selection mode.
/// On Windows, both Auto and Wasapi return the default WASAPI host.
pub fn get_host(selection: HostSelection) -> Result<Host, DouxError> {
    match selection {
        HostSelection::Auto | HostSelection::Wasapi => {
            for host_id in cpal::available_hosts() {
                if host_id.name().to_lowercase().contains("wasapi") {
                    if let Ok(host) = cpal::host_from_id(host_id) {
                        return Ok(host);
                    }
                }
            }
            Ok(cpal::default_host())
        }
    }
}

/// Returns the preferred audio host (WASAPI on Windows).
pub fn preferred_host() -> Host {
    for host_id in cpal::available_hosts() {
        if host_id.name().to_lowercase().contains("wasapi") {
            if let Ok(host) = cpal::host_from_id(host_id) {
                return host;
            }
        }
    }
    cpal::default_host()
}

/// Returns the default output device.
pub fn default_output_device() -> Option<Device> {
    preferred_host().default_output_device()
}

/// Returns the default input device.
pub fn default_input_device() -> Option<Device> {
    preferred_host().default_input_device()
}

/// Runs Windows-specific audio diagnostics.
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

    results
}
