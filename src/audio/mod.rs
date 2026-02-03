//! Audio device enumeration and stream creation utilities.
//!
//! Provides functions to list available audio devices and create audio streams
//! with specific configurations.

mod common;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub use common::{AudioDeviceInfo, AudioHostInfo, DiagnosticResult, DiagnosticStatus};

#[cfg(target_os = "linux")]
pub use linux::HostSelection;
#[cfg(target_os = "macos")]
pub use macos::HostSelection;
#[cfg(target_os = "windows")]
pub use windows::HostSelection;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

use crate::error::DouxError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SupportedStreamConfig};

/// Gets an audio host by selection mode.
pub fn get_host(selection: HostSelection) -> Result<Host, DouxError> {
    platform::get_host(selection)
}

/// Returns the preferred audio host for the current platform.
pub fn preferred_host() -> Host {
    platform::preferred_host()
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

/// Lists all available output audio devices.
pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
    let host = preferred_host();
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

/// Lists all available input audio devices.
pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
    let host = preferred_host();
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

/// Finds an output device by index or partial name match.
///
/// If `spec` parses as a number, returns the device at that index.
/// Otherwise, performs a case-insensitive substring match on device names.
pub fn find_output_device(spec: &str) -> Option<Device> {
    let host = preferred_host();
    let devices = host.output_devices().ok()?;
    find_device_impl(devices, spec)
}

/// Finds an input device by index or partial name match.
pub fn find_input_device(spec: &str) -> Option<Device> {
    let host = preferred_host();
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
        d.description()
            .map(|desc| desc.name().to_lowercase().contains(&spec_lower))
            .unwrap_or(false)
    })
}

/// Returns the default output device.
pub fn default_output_device() -> Option<Device> {
    platform::default_output_device()
}

/// Returns the default input device.
pub fn default_input_device() -> Option<Device> {
    platform::default_input_device()
}

/// Gets the default output config for a device.
pub fn default_output_config(device: &Device) -> Option<SupportedStreamConfig> {
    device.default_output_config().ok()
}

/// Returns true if the preferred audio host is JACK.
/// JACK enforces its own buffer size, so user-specified buffer sizes should be ignored.
pub fn is_jack_host() -> bool {
    preferred_host().id().name().to_lowercase().contains("jack")
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
    platform::run_diagnostics(&hosts)
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
