//! Audio device enumeration and stream creation utilities.
//!
//! Provides functions to list available audio devices and create audio streams
//! with specific configurations.

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
