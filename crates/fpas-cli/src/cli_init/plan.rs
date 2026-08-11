//! Complete in-memory file plans for init scaffolds.

use std::path::{Path, PathBuf};

use crate::cli_input::{InitCliConfig, InitKind};

use super::{naming::pascal_identifier, templates};

/// One file that must exist with exact UTF-8 content after initialization.
pub(super) struct PlannedFile {
    pub(super) path: PathBuf,
    pub(super) content: String,
}

/// Fully resolved, side-effect-free scaffold plan.
pub(super) struct ScaffoldPlan {
    pub(super) cwd: PathBuf,
    pub(super) root: PathBuf,
    pub(super) kind: InitKind,
    pub(super) name: String,
    pub(super) manifest: PathBuf,
    pub(super) files: Vec<PlannedFile>,
}

/// Builds all paths and contents without accessing the filesystem.
pub(super) fn build(config: &InitCliConfig) -> ScaffoldPlan {
    let identifier = pascal_identifier(&config.name);
    let template = match config.kind {
        InitKind::Project => templates::project(&config.name, &identifier),
        InitKind::Library => {
            let unit = config.library_unit.as_deref().unwrap_or(&identifier);
            templates::library(&config.name, unit)
        }
        InitKind::Workspace => templates::workspace(&config.name, &identifier),
    };
    let files = template
        .files
        .into_iter()
        .map(|entry| PlannedFile {
            path: config.root.join(&entry.relative_path),
            content: entry.content,
        })
        .collect();

    ScaffoldPlan {
        cwd: config.cwd.clone(),
        root: config.root.clone(),
        kind: config.kind,
        name: config.name.clone(),
        manifest: template.manifest,
        files,
    }
}

/// Returns a stable display path relative to the invocation directory when possible.
pub(super) fn display_path(path: &Path, cwd: &Path) -> String {
    let display = path.strip_prefix(cwd).unwrap_or(path);
    let text = display.to_string_lossy();
    if text.is_empty() {
        ".".to_string()
    } else {
        text.replace('\\', "/")
    }
}
