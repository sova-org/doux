//! Configuration types for the Doux audio engine.

use std::path::PathBuf;

/// Configuration for the Doux audio engine.
#[derive(Debug, Clone)]
pub struct DouxConfig {
    /// Output device specification (name or index). None uses system default.
    pub output_device: Option<String>,
    /// Input device specification (name or index). None uses system default.
    pub input_device: Option<String>,
    /// Number of output channels (will be clamped to device maximum).
    pub channels: u16,
    /// Paths to sample directories for lazy loading.
    pub sample_paths: Vec<PathBuf>,
    /// Audio buffer size in samples. None uses system default.
    pub buffer_size: Option<u32>,
}

impl Default for DouxConfig {
    fn default() -> Self {
        Self {
            output_device: None,
            input_device: None,
            channels: 2,
            sample_paths: Vec::new(),
            buffer_size: None,
        }
    }
}

impl DouxConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output_device(mut self, device: impl Into<String>) -> Self {
        self.output_device = Some(device.into());
        self
    }

    pub fn with_input_device(mut self, device: impl Into<String>) -> Self {
        self.input_device = Some(device.into());
        self
    }

    pub fn with_channels(mut self, channels: u16) -> Self {
        self.channels = channels;
        self
    }

    pub fn with_sample_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.sample_paths.push(path.into());
        self
    }

    pub fn with_sample_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.sample_paths.extend(paths);
        self
    }

    pub fn with_buffer_size(mut self, size: u32) -> Self {
        self.buffer_size = Some(size);
        self
    }
}
