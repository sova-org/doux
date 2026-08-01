//! Name resolution as a query, off the audio thread.

use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::event::effective_sample_name;
use crate::patch::PatchRegistry;
use crate::sampling::SampleIndex;
use crate::types::Source;

/// What a bare sound name plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Builtin,
    Patch,
    Sample,
    Unresolved,
}

/// Resolves sound names off the audio thread, so hosts can classify a name for
/// completion, validation, or scheduling without touching the Engine.
///
/// Holds the same two handles the Engine resolves against, so an Engine rebuilt
/// for a device change does not invalidate it.
#[derive(Clone)]
pub struct SourceResolver {
    patches: Arc<PatchRegistry>,
    sample_index: Arc<ArcSwap<SampleIndex>>,
}

impl SourceResolver {
    pub fn new(patches: Arc<PatchRegistry>, sample_index: Arc<ArcSwap<SampleIndex>>) -> Self {
        Self {
            patches,
            sample_index,
        }
    }

    /// INVARIANT: same precedence as dispatch in `Engine::process_event`.
    /// Builtin and patch test the bare name; only the folder takes the bank suffix.
    pub fn resolve(&self, name: &str, bank: Option<&str>) -> SourceKind {
        if name.parse::<Source>().is_ok() {
            return SourceKind::Builtin;
        }
        if self.patches.get(name).is_some_and(|e| e.is_source()) {
            return SourceKind::Patch;
        }
        let folder = effective_sample_name(name, bank);
        if self.sample_index.load().has_folder(&folder) {
            return SourceKind::Sample;
        }
        SourceKind::Unresolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampling::{SampleData, SampleEntry};
    use crate::{Engine, EngineConfig};
    use std::path::PathBuf;

    fn entry(name: &str) -> SampleEntry {
        SampleEntry {
            path: Arc::new(PathBuf::from(format!("/nonexistent/{name}.wav"))),
            name: name.into(),
        }
    }

    // The query and dispatch must not drift: every name the resolver calls
    // playable has to reach a voice, and Unresolved has to be dropped.
    #[test]
    fn resolve_agrees_with_dispatch() {
        let mut engine = Engine::new(EngineConfig::native(48_000.0, 2));

        // Source patch (zero inputs), installed like any host would.
        let mut g = arf::graph::Graph::new();
        let nf = g.control(arf::graph::NOTEFREQ_LANE as u32);
        let osc = g.ugen(arf::ugen::lookup("saw").unwrap(), vec![nf]);
        g.set_outputs(vec![osc]);
        let json = serde_json::to_string(&g).unwrap();
        engine
            .patch_registry()
            .install_graph("blip", &json, 48_000.0)
            .unwrap();

        // One plain folder and one only reachable through `bank`. Both are put
        // in the registry so dispatch resolves them here, not on a loader thread.
        engine.set_sample_index(vec![entry("break/0"), entry("break_hi/0")]);
        let data = Arc::new(SampleData::new(vec![0.0; 512], 1, 261.626));
        engine
            .sample_registry()
            .insert("break/0".to_string(), Arc::clone(&data));
        engine
            .sample_registry()
            .insert("break_hi/0".to_string(), data);

        let resolver = SourceResolver::new(
            Arc::clone(engine.patch_registry()),
            engine.sample_index_handle(),
        );

        let cases = [
            ("saw", None, SourceKind::Builtin),
            ("blip", None, SourceKind::Patch),
            ("break", None, SourceKind::Sample),
            ("break", Some("hi"), SourceKind::Sample),
            // The bank suffix is part of the folder name, so a wrong bank misses.
            ("break", Some("nope"), SourceKind::Unresolved),
            ("nosuchthing", None, SourceKind::Unresolved),
            // A folder never shadows a builtin of the same name.
            ("blip", Some("hi"), SourceKind::Patch),
        ];

        for (name, bank, expected) in cases {
            assert_eq!(
                resolver.resolve(name, bank),
                expected,
                "resolve({name}, {bank:?})"
            );
            let event = match bank {
                Some(b) => format!("sound/{name}/bank/{b}"),
                None => format!("sound/{name}"),
            };
            assert_eq!(
                engine.evaluate(&event).is_some(),
                expected != SourceKind::Unresolved,
                "dispatch {event}"
            );
            engine.panic();
        }
    }
}
