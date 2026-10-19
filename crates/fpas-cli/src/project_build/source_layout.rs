//! One canonical source tree shared by project debugging and published images.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::PROJECT_PATH_INVALID;

use crate::cli_output::CliFailure;

pub(crate) struct SourceLayout {
    pub(crate) root: PathBuf,
    pub(crate) paths: Vec<String>,
}

impl SourceLayout {
    /// Keep the project root when possible; widen it to contain selected sources.
    /// Different filesystem volumes have no shared root and retain legacy labels.
    pub(crate) fn derive(
        preferred_root: &Path,
        sources: &[PathBuf],
        selected: &[PathBuf],
    ) -> Result<Option<Self>, CliFailure> {
        let mut root = preferred_root.canonicalize().map_err(|error| {
            CliFailure::new(
                PROJECT_PATH_INVALID,
                format!(
                    "Cannot resolve source root `{}`: {error}",
                    preferred_root.display()
                ),
            )
        })?;
        let sources = sources
            .iter()
            .map(|path| {
                path.canonicalize()
                    .map_err(|error| super::source_read_failure(path, &error))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for path in selected {
            let path = path
                .canonicalize()
                .map_err(|error| super::source_read_failure(path, &error))?;
            while !path.starts_with(&root) {
                if !root.pop() {
                    return Ok(None);
                }
            }
        }
        let mut names = HashSet::new();
        let mut paths = sources
            .iter()
            .map(|path| {
                path.strip_prefix(&root).ok().map(|relative| {
                    let name = relative.to_string_lossy().replace('\\', "/");
                    names.insert(name.clone());
                    name
                })
            })
            .collect::<Vec<_>>();
        // Unselected graph entries still reserve source IDs while compiling,
        // but do not appear in the linked image or determine its source root.
        for (index, path) in paths.iter_mut().enumerate() {
            if path.is_none() {
                let mut name = format!("unlinked/source-{index}.fpas");
                while !names.insert(name.clone()) {
                    name.push('_');
                }
                *path = Some(name);
            }
        }
        let paths = paths.into_iter().flatten().collect();
        Ok(Some(Self { root, paths }))
    }
}
