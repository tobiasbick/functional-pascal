//! Observable results of explicit record functions, closures and optional handlers.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`.

use super::{repo_root, support};

#[test]
fn ordinary_record_examples_preserve_snapshots_and_handlers() {
    let root = repo_root();
    for (name, expected) in [
        ("receiver_closures", "15\n17\nBase: 3\n"),
        ("optional_handlers", "no handler\nclicked 1\nno handler\n"),
        (
            "record_accessors",
            "Stored value: 10\nSetter received: 20\nStored value after setter: 10\n",
        ),
    ] {
        let path = format!("examples/pascal/functions/{name}.fpas");
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[
                "run".into(),
                "--std-lib".into(),
                root.join("lib").to_string_lossy().into_owned(),
                path.clone(),
            ],
            &root,
        );
        assert_eq!(exit, 0, "{path}: {stderr}");
        assert!(stderr.is_empty(), "{path}: {stderr}");
        assert_eq!(stdout.replace("\r\n", "\n"), expected, "{path}");
    }
}
