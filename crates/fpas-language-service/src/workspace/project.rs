//! Loaded project ownership and visibility queries.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use fpas_project::{LibraryExportPolicy, LoadedProject, ProjectKind, SourceOrigin};

use crate::document::normalized_path;

/// One loaded project and its resolved source ownership.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    manifest_path: PathBuf,
    project: LoadedProject,
    source_paths: HashSet<PathBuf>,
    owned_source_paths: HashSet<PathBuf>,
    source_origins: HashMap<PathBuf, SourceOrigin>,
}

impl ProjectContext {
    pub(super) fn new(manifest_path: &Path, project: LoadedProject) -> Self {
        let manifest_path = normalized_path(manifest_path);
        let source_origins = project
            .link_meta
            .source_origins
            .iter()
            .map(|(source, origin)| (normalized_path(source), origin.clone()))
            .collect::<HashMap<_, _>>();
        let mut source_paths = project
            .source_files
            .iter()
            .map(|source| normalized_path(source))
            .collect::<HashSet<_>>();
        if let Some(main) = &project.main {
            source_paths.insert(normalized_path(main));
        }
        let owned_source_paths = source_paths
            .iter()
            .filter(|source| {
                project
                    .main
                    .as_ref()
                    .is_some_and(|main| normalized_path(main) == source.as_path())
                    || source_origins
                        .get(*source)
                        .is_none_or(|origin| *origin == SourceOrigin::Own)
            })
            .cloned()
            .collect();
        Self {
            manifest_path,
            project,
            source_paths,
            owned_source_paths,
            source_origins,
        }
    }

    /// Returns the normalized `.fpasprj` path.
    #[must_use]
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    /// Returns the declared project name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.project.name
    }

    /// Returns the declared project kind.
    #[must_use]
    pub fn kind(&self) -> ProjectKind {
        self.project.kind
    }

    /// Returns the optional program entry source.
    #[must_use]
    pub fn main(&self) -> Option<&Path> {
        self.project.main.as_deref()
    }

    /// Returns the complete resolved user-unit and dependency source list.
    #[must_use]
    pub fn source_files(&self) -> &[PathBuf] {
        &self.project.source_files
    }

    /// Returns whether this project owns or consumes the given source path.
    #[must_use]
    pub fn contains_source(&self, path: &Path) -> bool {
        self.source_paths.contains(&normalized_path(path))
    }

    pub(crate) fn owns_source(&self, path: &Path) -> bool {
        self.owned_source_paths.contains(&normalized_path(path))
    }

    pub(crate) fn loaded(&self) -> &LoadedProject {
        &self.project
    }

    pub(crate) fn all_source_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.source_paths.iter().cloned().collect::<Vec<_>>();
        paths.sort();
        paths
    }

    pub(crate) fn source_visible_from(
        &self,
        from: &Path,
        candidate: &Path,
        candidate_unit: &str,
    ) -> bool {
        let from_origin = self.source_origin(from);
        let candidate_origin = self.source_origin(candidate);
        if from_origin == candidate_origin {
            return true;
        }
        let SourceOrigin::Library(library) = candidate_origin else {
            return false;
        };
        match self.project.link_meta.export_policy_for_library(&library) {
            LibraryExportPolicy::AllUnits => true,
            LibraryExportPolicy::ListedUnits(units) => {
                units.contains(&candidate_unit.to_ascii_lowercase())
            }
        }
    }

    fn source_origin(&self, path: &Path) -> SourceOrigin {
        self.source_origins
            .get(&normalized_path(path))
            .cloned()
            .unwrap_or(SourceOrigin::Own)
    }
}
