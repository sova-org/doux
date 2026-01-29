//! Error types for the Doux audio engine.

use std::fmt;

/// Errors that can occur when working with the Doux audio engine.
#[derive(Debug)]
pub enum DouxError {
    /// The specified audio device was not found.
    DeviceNotFound(String),
    /// No default audio device is available.
    NoDefaultDevice,
    /// Failed to create an audio stream.
    StreamCreationFailed(String),
    /// The requested channel count is invalid.
    InvalidChannelCount(u16),
    /// Failed to get device configuration.
    DeviceConfigError(String),
    /// The specified audio host was not found.
    HostNotFound(String),
    /// No audio devices available on the specified host.
    NoDevicesAvailable { host: String },
    /// Permission denied when accessing audio.
    PermissionDenied(String),
}

impl fmt::Display for DouxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DouxError::DeviceNotFound(name) => {
                write!(f, "audio device not found: {name}")
            }
            DouxError::NoDefaultDevice => {
                write!(f, "no default audio device available")
            }
            DouxError::StreamCreationFailed(msg) => {
                write!(f, "failed to create audio stream: {msg}")
            }
            DouxError::InvalidChannelCount(count) => {
                write!(f, "invalid channel count: {count}")
            }
            DouxError::DeviceConfigError(msg) => {
                write!(f, "device configuration error: {msg}")
            }
            DouxError::HostNotFound(name) => {
                write!(f, "audio host not found: {name}")
            }
            DouxError::NoDevicesAvailable { host } => {
                write!(f, "no audio devices available on host: {host}")
            }
            DouxError::PermissionDenied(msg) => {
                write!(f, "permission denied: {msg}")
            }
        }
    }
}

impl std::error::Error for DouxError {}
