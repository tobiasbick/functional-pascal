//! Composition preserves source order, canonical identity, and library metadata.

use super::{
    LibraryExportPolicy, LoadedProject, ProjectKind, ProjectLinkMeta, SourceOrigin, TestManifest,
    merge_standard_library, normalized_path,
};
use std::path::{Path, PathBuf};

fn project(paths: &[&str]) -> LoadedProject {
    LoadedProject {
        name: "fixture".to_owned(),
        kind: ProjectKind::Library,
        main: None,
        source_files: paths.iter().map(PathBuf::from).collect(),
        warnings: Vec::new(),
        link_meta: ProjectLinkMeta::default(),
        export_policy_for_dependents: LibraryExportPolicy::AllUnits,
        test_manifest: TestManifest::default(),
    }
}

#[test]
fn composition_deduplicates_normalized_paths_and_preserves_first_source_order() {
    let existing = ".temp-data/compose-fixture/lib/../first.fpas";
    let mut target = project(&[existing, ".temp-data/compose-fixture/own.fpas"]);
    let mut library = project(&[
        ".temp-data/compose-fixture/first.fpas",
        ".temp-data/compose-fixture/second.fpas",
        ".temp-data/compose-fixture/second.fpas",
    ]);
    let second = normalized_path(Path::new(".temp-data/compose-fixture/second.fpas"));
    library
        .link_meta
        .trusted_standard_library_sources
        .insert(second.clone());
    let manifest = Path::new(".temp-data/compose-fixture/stdlib.fpasprj");
    merge_standard_library(&mut target, manifest, &library);
    assert_eq!(
        target.source_files,
        vec![
            PathBuf::from(existing),
            PathBuf::from(".temp-data/compose-fixture/own.fpas"),
            second.clone()
        ]
    );
    for source in [normalized_path(Path::new(existing)), second.clone()] {
        assert_eq!(
            target.link_meta.source_origins.get(&source),
            Some(&SourceOrigin::Library(normalized_path(manifest)))
        );
    }
    assert!(
        target
            .link_meta
            .trusted_standard_library_sources
            .contains(&second)
    );
    assert_eq!(
        target
            .link_meta
            .library_export_policies
            .get(&normalized_path(manifest)),
        Some(&LibraryExportPolicy::AllUnits)
    );
}

#[test]
fn composition_retains_existing_duplicates_and_is_idempotent() {
    let mut target = project(&[
        ".temp-data/compose-fixture/own.fpas",
        ".temp-data/compose-fixture/own.fpas",
    ]);
    let library = project(&[".temp-data/compose-fixture/new.fpas"]);
    let manifest = Path::new(".temp-data/compose-fixture/stdlib.fpasprj");
    merge_standard_library(&mut target, manifest, &library);
    let first = target.clone();
    merge_standard_library(&mut target, manifest, &library);
    assert_eq!(target, first);
    assert_eq!(target.source_files.len(), 3);
}

#[test]
fn empty_standard_library_preserves_user_sources() {
    let mut target = project(&[".temp-data/compose-fixture/own.fpas"]);
    let sources = target.source_files.clone();
    merge_standard_library(
        &mut target,
        Path::new(".temp-data/compose-fixture/stdlib.fpasprj"),
        &project(&[]),
    );
    assert_eq!(target.source_files, sources);
    assert!(target.link_meta.source_origins.is_empty());
}
