//! Program-image publication ordering regressions.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;

use fpas_program::Digest;
use fpas_project::{ProjectLinkMeta, build_unit_graph_for_program};

use super::{
    ProgramArtifactTarget, build_program_artifact, build_program_artifact_before_publish, source,
};
use crate::BuildOptions;

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-program-publication-{}-{id}",
        std::process::id()
    ))
}

fn graph_for(main: &Path, source_files: &[PathBuf]) -> fpas_project::UnitGraph {
    build_unit_graph_for_program(main, source_files, &ProjectLinkMeta::default())
        .expect("program unit graph")
}

#[test]
fn stale_build_cannot_replace_a_newer_program_image() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let main = root.join("main.fpas");
    let artifact = root.join("race.fpascp");
    let source_paths = vec!["main.fpas".to_string()];
    let old_source = b"program Race; begin end.".to_vec();
    fs::write(&main, &old_source).expect("old main source");
    let old_graph = graph_for(&main, &[]);

    let old_artifact = artifact.clone();
    let old_source_paths = source_paths.clone();
    let (old_ready_tx, old_ready_rx) = mpsc::channel();
    let (release_old_tx, release_old_rx) = mpsc::channel();
    let old_builder = std::thread::spawn(move || {
        build_program_artifact_before_publish(
            &old_graph,
            ProgramArtifactTarget {
                path: &old_artifact,
                source: &old_source,
                source_paths: &old_source_paths,
            },
            &BuildOptions::default(),
            || {
                old_ready_tx.send(()).expect("old build ready receiver");
                release_old_rx.recv().expect("old build release");
            },
        )
    });

    old_ready_rx.recv().expect("old build reached publication");
    let new_source = b"program Race; begin end.\n".to_vec();
    fs::write(&main, &new_source).expect("new main source");
    let new_graph = graph_for(&main, &[]);
    build_program_artifact(
        &new_graph,
        ProgramArtifactTarget {
            path: &artifact,
            source: &new_source,
            source_paths: &source_paths,
        },
        &BuildOptions::default(),
    )
    .expect("new build publication");
    release_old_tx.send(()).expect("release old build");
    let old_result = old_builder.join().expect("old builder thread");

    let published = fpas_program::decode(&fs::read(&artifact).expect("published image"))
        .expect("valid published image");
    assert_eq!(
        (old_result.is_err(), published.identity().source_hash),
        (true, Digest::of(&new_source))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn changed_unit_is_rejected_by_final_snapshot_validation() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temporary directory");
    let main = root.join("main.fpas");
    let unit = root.join("unit.fpas");
    let main_source = b"program Race; uses Race.Work; begin end.";
    fs::write(&main, main_source).expect("main source");
    fs::write(
        &unit,
        "unit Race.Work; public function Value(): integer; begin return 1 end;",
    )
    .expect("old unit source");
    let graph = graph_for(&main, std::slice::from_ref(&unit));

    fs::write(
        &unit,
        "unit Race.Work; public function Value(): integer; begin return 2 end;",
    )
    .expect("new unit source");

    let error =
        source::ensure_current(&graph, Digest::of(main_source)).expect_err("changed unit snapshot");
    assert!(error.to_string().contains("changed during the build"));
    fs::remove_dir_all(root).ok();
}
