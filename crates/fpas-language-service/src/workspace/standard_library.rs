//! Source-standard-library composition for editor analysis.

use std::path::Path;

use fpas_project::{
    LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin, TestManifest,
    load_standard_library_project,
};

use super::ProjectContext;
use crate::document::normalized_path;

#[derive(Debug, Clone)]
pub(crate) struct StandardLibraryContext {
    project: ProjectContext,
}

impl StandardLibraryContext {
    pub(crate) fn load(root: &Path) -> Result<Self, String> {
        let project = load_standard_library_project(root)?;
        Ok(Self {
            project: ProjectContext::new(&root.join("stdlib.fpasprj"), project),
        })
    }

    pub(crate) fn compose(&self, project: &ProjectContext) -> ProjectContext {
        if project.manifest_path() == self.project.manifest_path() {
            return project.clone();
        }

        let mut loaded = project.loaded().clone();
        merge_standard_library(
            &mut loaded,
            self.project.manifest_path(),
            self.project.loaded(),
        );
        ProjectContext::new(project.manifest_path(), loaded)
    }

    pub(crate) fn compose_loose(
        &self,
        path: &Path,
        compilation_kind: ProjectKind,
    ) -> ProjectContext {
        let path = normalized_path(path);
        let is_program = compilation_kind == ProjectKind::Program;
        let mut link_meta = ProjectLinkMeta::default();
        link_meta
            .source_origins
            .insert(path.clone(), SourceOrigin::Own);
        let mut loaded = LoadedProject {
            name: String::from("loose-editor-document"),
            kind: compilation_kind,
            main: is_program.then(|| path.clone()),
            source_files: if is_program {
                Vec::new()
            } else {
                vec![path.clone()]
            },
            warnings: Vec::new(),
            link_meta,
            export_policy_for_dependents: LibraryExportPolicy::AllUnits,
            test_manifest: TestManifest::default(),
        };
        merge_standard_library(
            &mut loaded,
            self.project.manifest_path(),
            self.project.loaded(),
        );
        ProjectContext::new(&path, loaded)
    }
}

fn merge_standard_library(
    target: &mut LoadedProject,
    manifest_path: &Path,
    standard_library: &LoadedProject,
) {
    let manifest_path = normalized_path(manifest_path);
    target.link_meta.library_export_policies.insert(
        manifest_path.clone(),
        standard_library.export_policy_for_dependents.clone(),
    );
    target.link_meta.trusted_standard_library_sources.extend(
        standard_library
            .link_meta
            .trusted_standard_library_sources
            .iter()
            .map(|source| normalized_path(source)),
    );

    for source in &standard_library.source_files {
        let source = normalized_path(source);
        if !target
            .source_files
            .iter()
            .any(|existing| normalized_path(existing) == source)
        {
            target.source_files.push(source.clone());
        }
        target
            .link_meta
            .source_origins
            .insert(source, SourceOrigin::Library(manifest_path.clone()));
    }
}
