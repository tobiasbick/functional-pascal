//! Project link-context discovery for `fpas test`.
//!
//! Spec: [`docs/pascal/program-structure/projects.md`](../../../docs/pascal/program-structure/projects.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::hooks;
use super::run::LinkContext;
use crate::cli_paths::{PROJECT_FILE_EXTENSION, has_extension};
use fpas_project as project;

/// Caches loaded project link contexts while a test run walks many files.
#[derive(Default)]
pub(super) struct LinkContextCache {
    contexts: HashMap<PathBuf, LinkContext>,
    unscoped: Option<LinkContext>,
    standard_library: Option<Arc<project::StandardLibrary>>,
}

impl LinkContextCache {
    /// Creates an empty cache for one `fpas test` invocation.
    pub(super) fn new(standard_library: Option<Arc<project::StandardLibrary>>) -> Self {
        Self {
            contexts: HashMap::new(),
            unscoped: None,
            standard_library,
        }
    }

    /// Returns the enclosing project context for `path`, loading each project at most once.
    pub(super) fn context_for_test(&mut self, path: &Path) -> Result<Option<LinkContext>, String> {
        let Some(project_file) = find_enclosing_project(path)? else {
            if let Some(context) = &self.unscoped {
                return Ok(Some(context.clone()));
            }
            let Some(standard_library) = &self.standard_library else {
                return Ok(None);
            };
            let program_graph = project::prepare_program_unit_graph(
                &[],
                &project::ProjectLinkMeta::default(),
                Some(standard_library),
            )?;
            let context = LinkContext {
                source_files: Vec::new(),
                program_graph: Arc::new(program_graph),
                test_manifest: project::TestManifest::default(),
                hooks: hooks::TestHooks::default(),
            };
            self.unscoped = Some(context.clone());
            return Ok(Some(context));
        };
        if let Some(context) = self.contexts.get(&project_file) {
            return Ok(Some(context.clone()));
        }

        let loaded = project::load_project(&project_file)?;
        // Test entry programs are run individually and are never linkable unit sources.
        let source_files = loaded
            .source_files
            .into_iter()
            .filter(|path| !project::is_test_source_file(path))
            .collect::<Vec<_>>();
        let hooks = hooks::discover_test_hooks(&source_files)?;
        let program_graph = project::prepare_program_unit_graph(
            &source_files,
            &loaded.link_meta,
            self.standard_library.as_deref(),
        )?;
        let context = LinkContext {
            source_files,
            program_graph: Arc::new(program_graph),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn context_excludes_test_entry_programs_from_linkable_sources() {
        let dir = create_temp_dir("fpas-link-context-sources");
        let project_file = dir.join("suite.fpasprj");
        let first_test = dir.join("first_test.fpas");
        let second_test = dir.join("second_test.fpas");
        let helper = dir.join("fixture.fpas");

        write_text(
            &project_file,
            "[project]\nname = \"suite\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
        );
        write_text(&first_test, "program First;\nbegin end.");
        write_text(&second_test, "program Second;\nbegin end.");
        write_text(&helper, "unit Tests.Fixture;\n");

        let mut contexts = LinkContextCache::new(None);
        let context = contexts
            .context_for_test(&first_test)
            .expect("test project must load")
            .expect("test project must provide a link context");
        let second_context = contexts
            .context_for_test(&second_test)
            .expect("test project must stay loaded")
            .expect("test project must keep its link context");

        assert_eq!(context.source_files, vec![helper]);
        assert!(Arc::ptr_eq(
            &context.program_graph,
            &second_context.program_graph
        ));
        std::fs::remove_dir_all(dir).ok();
    }
}
