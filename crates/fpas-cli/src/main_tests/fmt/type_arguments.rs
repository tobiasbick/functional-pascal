//! Rejected type applications must not be rewritten by the formatter CLI.
//! See `docs/pascal/language/types/generics.md`.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn rejected_type_applications_preserve_source_and_report_the_canonical_form() {
    for application in ["Result of integer, string", "Result<integer, string>"] {
        let directory = create_temp_dir("fmt-type-arguments");
        let path = directory.join("type-arguments.fpas");
        let source = format!("program T; begin const Value: {application} := Ok(1); end.");
        write_text(&path, &source);
        let (code, _, stderr) = run_cli_args_and_capture_output(
            &["fmt".into(), path.to_string_lossy().into_owned()],
            &directory,
        );
        assert_eq!(code, 1, "{stderr}");
        assert!(
            stderr.contains("FP2001") && stderr.contains("Result of ("),
            "{stderr}"
        );
        assert_eq!(fs::read_to_string(&path).expect("preserved source"), source);
        fs::remove_dir_all(directory).expect("remove fixture");
    }
}
