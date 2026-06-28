//! Project link-context discovery for `fpas test`.
//!
//! Spec: [`docs/pascal/program-structure/projects.md`](../../../docs/pascal/program-structure/projects.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::hooks;
use super::run::LinkContext;
use crate::cli_paths::{PROJECT_FILE_EXTENSION, has_extension};
use fpas_project as project;

/// Caches loaded project link contexts while a test run walks many files.
#[derive(Default)]
pub(super) struct LinkContextCache {
    contexts: HashMap<PathBuf, LinkContext>,
}

impl LinkContextCache {
    /// Creates an empty cache for one `fpas test` invocation.
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Returns the enclosing project context for `path`, loading each project at most once.
    pub(super) fn context_for_test(&mut self, path: &Path) -> Result<Option<LinkContext>, String> {
        let Some(project_file) = find_enclosing_project(path)? else {
            return Ok(None);
        };
        if let Some(context) = self.contexts.get(&project_file) {
            return Ok(Some(context.clone()));
        }

        let loaded = project::load_project(&project_file)?;
        let hooks = hooks::discover_test_hooks(&loaded.source_files)?;
        let context = LinkContext {
            source_files: loaded.source_files,
            link_meta: loaded.link_meta,
            test_manifest: loaded.test_manifest,
            hooks,
        };
        self.contexts.insert(project_file, context.clone());
        Ok(Some(context))
    }
}

fn find_enclosing_project(start: &Path) -> Result<Option<PathBuf>, String> {
    let mut dir = start
        .parent()
        .ok_or_else(|| {
            format!(
                "Cannot resolve enclosing project for `{}`.",
                start.display()
            )
        })?
        .to_path_buf();
    loop {
        let mut candidates = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file() && has_extension(&path, PROJECT_FILE_EXTENSION) {
                    candidates.push(path);
                }
            }
        }
        candidates.sort();
        match candidates.len() {
            0 => {}
            1 => return Ok(Some(candidates.remove(0))),
            _ => {
                let entries = candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "Found multiple `.fpasprj` files in `{}`: {entries}.\n  help: Keep one project manifest per directory or pass an explicit `.fpasprj` path.",
                    dir.display()
                ));
            }
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}
