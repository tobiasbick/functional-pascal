//! JSON query contracts at the actual CLI dispatch boundary.

mod rename;

use super::*;
use crate::main_tests::support::run_cli_args_and_capture_output;
use serde_json::{Value, json};

fn query(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let mut argv = vec!["query".to_owned()];
    argv.extend(args.iter().map(|arg| (*arg).to_owned()));
    if !args.is_empty() {
        argv.extend([
            "--std-lib".into(),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../lib")
                .to_string_lossy()
                .into_owned(),
        ]);
    }
    run_cli_args_and_capture_output(&argv, cwd)
}

fn report(cwd: &Path, args: &[&str]) -> Value {
    let (code, stdout, stderr) = query(cwd, args);
    assert_eq!(code, 0, "{args:?}: {stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    serde_json::from_str(&stdout).expect("exactly one JSON document")
}
