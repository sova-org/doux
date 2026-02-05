//! Sample storage, loading, and playback.

mod cursor;
#[cfg(feature = "native")]
mod decode;
#[cfg(feature = "native")]
mod loader;
#[cfg(feature = "native")]
mod registry;
mod sample;
#[cfg(feature = "native")]
mod source;

pub use cursor::Cursor;
pub use sample::SampleEntry;
#[cfg(not(feature = "native"))]
pub use sample::{FileSource, SampleInfo, SamplePool};
pub use sample::{WebSampleInfo, WebSampleSource};

#[cfg(feature = "native")]
pub use decode::{decode_sample_file, decode_sample_head, scan_samples_dir, HEAD_FRAMES};
#[cfg(feature = "native")]
pub use loader::SampleLoader;
#[cfg(feature = "native")]
pub use registry::{SampleData, SampleRegistry};
#[cfg(feature = "native")]
pub use source::RegistrySample;
