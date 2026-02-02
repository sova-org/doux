//! macOS-specific audio implementation (CoreAudio).

use super::common::{AudioHostInfo, DiagnosticResult};
use crate::error::DouxError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host};

/// Host selection mode for macOS.
/// CoreAudio is the only option on macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HostSelection {
    #[default]
    Auto,
}

impl std::str::FromStr for HostSelection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" | "coreaudio" => Ok(HostSelection::Auto),
            "jack" | "alsa" => Err(format!(
                "{s} is not available on macOS (only CoreAudio is supported)"
            )),
            _ => Err(format!("unknown host: {s} (use: auto)")),
        }
    }
}

impl std::fmt::Display for HostSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "auto")
    }
}

/// Gets an audio host by selection mode.
/// On macOS, always returns the default CoreAudio host.
pub fn get_host(_selection: HostSelection) -> Result<Host, DouxError> {
    Ok(cpal::default_host())
}

/// Returns the preferred audio host (CoreAudio on macOS).
pub fn preferred_host() -> Host {
    cpal::default_host()
}

/// Returns the default output device.
pub fn default_output_device() -> Option<Device> {
    cpal::default_host().default_output_device()
}

/// Returns the default input device.
pub fn default_input_device() -> Option<Device> {
    cpal::default_host().default_input_device()
}

/// Runs macOS-specific audio diagnostics.
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

    let host = cpal::default_host();
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
