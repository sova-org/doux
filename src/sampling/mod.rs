//! Sample storage, loading, and playback.

#[cfg(feature = "native")]
mod decode;
#[cfg(feature = "native")]
mod loader;
#[cfg(feature = "native")]
mod registry;
mod sample;

pub use sample::SampleEntry;
#[cfg(not(feature = "native"))]
pub use sample::{FileSource, SampleInfo, SamplePool};
pub use sample::{WebSampleInfo, WebSampleSource};

#[cfg(feature = "native")]
pub use decode::{decode_sample_file, scan_samples_dir};
#[cfg(feature = "native")]
pub use loader::SampleLoader;
#[cfg(feature = "native")]
pub use registry::{SampleData, SampleRegistry};
