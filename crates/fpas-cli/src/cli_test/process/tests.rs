use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use super::output::CappedBuffer;
use super::{WorkerFiles, run_worker_from_args};
use crate::test_support::create_temp_dir;

#[test]
fn capped_worker_output_reports_overflow() {
    let mut output = CappedBuffer::new(4);

    assert!(output.write_all(b"pass").is_ok());
    assert!(output.write_all(b"!").is_err());
    assert!(output.overflowed());
    assert_eq!(output.into_inner(), b"pass");
}

#[test]
fn public_arguments_do_not_enter_worker_mode() {
    assert_eq!(run_worker_from_args(&["test".to_string()]), None);
}

#[test]
fn worker_directory_skips_existing_entries_without_changing_them() {
    const CHILD_ROOT: &str = "FPAS_TEST_WORKER_COLLISION_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        // A fresh process makes the first allocation deterministic without sharing
        // the allocator counter or changing the parent test process environment.
        let root = PathBuf::from(root);
        assert_eq!(std::env::temp_dir(), root);
        let occupied_directory = root.join(format!("fpas-test-process-{}-1", std::process::id()));
        let occupied_file = root.join(format!("fpas-test-process-{}-2", std::process::id()));
        fs::create_dir(&occupied_directory).expect("occupied directory must be created");
        let marker = occupied_directory.join("previous-run.txt");
        fs::write(&marker, b"previous directory contents").expect("marker must be written");
        fs::write(&occupied_file, b"previous file contents")
            .expect("occupied file must be written");

        let files = WorkerFiles::create().expect("occupied names must not prevent a fresh worker");
        assert!(files.root.starts_with(&root));
        assert_ne!(files.root, occupied_directory);
        assert_ne!(files.root, occupied_file);
        assert!(
            fs::read_dir(&files.root)
                .expect("worker directory must exist")
                .next()
                .is_none()
        );
        files.cleanup();
        assert!(!files.root.exists());
        assert_eq!(
            fs::read(&marker).expect("marker must survive"),
            b"previous directory contents"
        );
        assert_eq!(
            fs::read(&occupied_file).expect("occupied file must survive"),
            b"previous file contents"
        );
        return;
    }

    let parent = create_temp_dir("worker-directory-collision");
    let root = parent.join("fixture");
    fs::create_dir(&root).expect("fixture directory must be newly created");
    let result = Command::new(std::env::current_exe().expect("test executable must exist"))
        .args([
            "--exact",
            "cli_test::process::tests::worker_directory_skips_existing_entries_without_changing_them",
            "--nocapture",
        ])
        .env(CHILD_ROOT, &root)
        .env("TMPDIR", &root)
        .env("TMP", &root)
        .env("TEMP", &root)
        .output();
    fs::remove_dir_all(&root).expect("owned fixture directory must be removed");
    fs::remove_dir(&parent).expect("empty fixture parent must be removed");
    let output = result.expect("collision fixture process must run");
    assert!(
        output.status.success(),
        "collision fixture failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn isolated_worker_request_preserves_effective_collection_configuration() {
    use super::{WorkerRequest, write_worker_inputs};
    use crate::cli_test::run::program::{PreparedProgram, RunOutput};
    use std::{num::NonZeroUsize, sync::Arc};
    let config = fpas_bytecode::CollectionSchedule {
        allocations: NonZeroUsize::new(13).unwrap(),
        mutations: NonZeroUsize::new(17).unwrap(),
        safe_points: NonZeroUsize::new(19).unwrap(),
    };
    let (program, errors) = fpas_parser::parse("program WorkerConfig; begin end.");
    assert!(errors.is_empty());
    let files = WorkerFiles::create().unwrap();
    let prepared = PreparedProgram {
        collection_schedule: config,
        test_path: files.root.join("worker_test.fpas"),
        executable: fpas_compiler::compile(&program).unwrap(),
        source_paths: None,
        sources: Arc::new(vec![]),
        script_override: None,
        manifest_override: None,
        display: "worker_test.fpas".into(),
        output: RunOutput::Test,
        show_output: false,
        scratch_dir: files.root.clone(),
    };
    write_worker_inputs(
        &files,
        &prepared,
        crate::cli_output::DiagnosticFormat::Text,
        false,
    )
    .unwrap();
    let bytes = fs::read(files.request()).unwrap();
    let request: WorkerRequest = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(request.collection_schedule, config);
    let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    invalid["collection_schedule"]["allocations"] = serde_json::json!(0);
    assert!(serde_json::from_value::<WorkerRequest>(invalid).is_err());
    files.cleanup();
}
