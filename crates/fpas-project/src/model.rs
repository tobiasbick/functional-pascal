use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Kind of `.fpasprj` project described in `docs/pascal/10-projects.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Executable project with a single `program` entry file.
    Program,
    /// Library project that contains only `unit` source files.
    Library,
    /// Test project: `*_test.fpas` programs discovered by `fpas test`, not `fpas run`.
    Test,
}

/// Which project contributed a merged source file during loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceOrigin {
    /// The root project being built or checked.
    Own,
    /// A transitive library dependency (canonical `.fpasprj` path).
    Library(PathBuf),
}

/// How a library project exposes units to external consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryExportPolicy {
    /// Every unit in the library may be imported by dependents.
    AllUnits,
    /// Only the listed unit names (case-insensitive) are importable across project boundaries.
    ListedUnits(HashSet<String>),
}

/// Per-file origins and per-library export rules used during linking.
///
/// Documentation: `docs/pascal/10-projects.md` (`[exports]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectLinkMeta {
    /// Maps each merged `.fpas` path to the project that owns it.
    pub source_origins: HashMap<PathBuf, SourceOrigin>,
    /// Export policy for each library `.fpasprj` merged as a dependency.
    pub library_export_policies: HashMap<PathBuf, LibraryExportPolicy>,
}

impl ProjectLinkMeta {
    /// Returns whether cross-project export rules should be enforced.
    pub fn enforces_export_rules(&self) -> bool {
        !self.source_origins.is_empty()
    }

    /// Origin for a source path; defaults to [`SourceOrigin::Own`] when unknown.
    pub fn origin_for_source(&self, source_path: &Path) -> SourceOrigin {
        self.source_origins
            .get(source_path)
            .cloned()
            .unwrap_or(SourceOrigin::Own)
    }

    /// Export policy for a library project path, defaulting to [`LibraryExportPolicy::AllUnits`].
    pub fn export_policy_for_library(&self, library_project: &Path) -> LibraryExportPolicy {
        self.library_export_policies
            .get(library_project)
            .cloned()
            .unwrap_or(LibraryExportPolicy::AllUnits)
    }
}

/// Resolved project metadata and source file set ready for linking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    /// Declared project kind.
    pub kind: ProjectKind,
    /// Main program file for executable projects.
    pub main: Option<PathBuf>,
    /// Validated user-unit source files included by the project.
    pub source_files: Vec<PathBuf>,
    /// Non-fatal loading warnings such as duplicate include entries.
    pub warnings: Vec<String>,
    /// Origins and library export policies for dependency-aware linking.
    pub link_meta: ProjectLinkMeta,
    /// How dependents may import units from this project when it is a library dependency.
    pub(crate) export_policy_for_dependents: LibraryExportPolicy,
}

impl LoadedProject {
    /// Export policy applied when this project is consumed as a library dependency.
    pub(crate) fn export_policy_for_dependents(&self) -> LibraryExportPolicy {
        self.export_policy_for_dependents.clone()
    }
}
