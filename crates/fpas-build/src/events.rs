//! Structured build activity used by deterministic tests and diagnostics.

/// One unit build action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvent {
    /// Canonical unit name, or the root program name.
    pub owner: String,
    /// Completed action.
    pub kind: BuildEventKind,
}

/// Stable build action category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEventKind {
    /// Source was parsed for a rebuild.
    Parsed,
    /// Public semantic interface was analyzed.
    InterfaceAnalyzed,
    /// Private implementation and routine bodies were analyzed.
    ImplementationAnalyzed,
    /// Relocatable bytecode was emitted.
    Compiled,
    /// A valid source-adjacent object was reused.
    SidecarReused,
    /// Objects were linked into an executable image.
    Relinked,
    /// A compatible compiled program image was reused.
    ProgramImageReused,
}

/// Aggregate counts derived from a build event stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildCounters {
    /// Parsed sources.
    pub parsed: usize,
    /// Analyzed interfaces.
    pub interface_analyzed: usize,
    /// Analyzed implementations.
    pub implementation_analyzed: usize,
    /// Compiled objects.
    pub compiled: usize,
    /// Reused sidecars.
    pub sidecar_reused: usize,
    /// Final links.
    pub relinked: usize,
    /// Reused compiled program images.
    pub program_image_reused: usize,
}

impl BuildCounters {
    /// Count the supplied structured events.
    #[must_use]
    pub fn from_events(events: &[BuildEvent]) -> Self {
        let mut counters = Self::default();
        for event in events {
            match event.kind {
                BuildEventKind::Parsed => counters.parsed += 1,
                BuildEventKind::InterfaceAnalyzed => counters.interface_analyzed += 1,
                BuildEventKind::ImplementationAnalyzed => {
                    counters.implementation_analyzed += 1;
                }
                BuildEventKind::Compiled => counters.compiled += 1,
                BuildEventKind::SidecarReused => counters.sidecar_reused += 1,
                BuildEventKind::Relinked => counters.relinked += 1,
                BuildEventKind::ProgramImageReused => counters.program_image_reused += 1,
            }
        }
        counters
    }
}
