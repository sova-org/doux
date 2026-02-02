//! Shared types for audio device enumeration.

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

/// Diagnostic result for audio setup.
#[derive(Debug, Clone)]
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
    pub fn ok(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Ok,
            message: message.to_string(),
        }
    }

    pub fn warn(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Warn,
            message: message.to_string(),
        }
    }

    pub fn error(label: &str, message: &str) -> Self {
        Self {
            label: label.to_string(),
            status: DiagnosticStatus::Error,
            message: message.to_string(),
        }
    }
}
