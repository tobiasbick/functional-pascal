//! CLI rejection and execution coverage for AP13.5 case arms.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn legacy_case_arms_are_rejected_without_rewriting_source() {
    let cwd = create_temp_dir("fmt-legacy-case");
    let path = cwd.join("legacy.fpas");
    let source = "program T; begin case 1 of 1: WriteLn('old'); end; end.";
    write_text(&path, source);
    let (code, _, stderr) =
        run_cli_args_and_capture_output(&["fmt".into(), path.to_string_lossy().into_owned()], &cwd);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("FP2001") && stderr.contains("when"),
        "{stderr}"
    );
    assert_eq!(fs::read_to_string(&path).expect("source retained"), source);
    fs::remove_dir_all(cwd).expect("remove fixture");
}

#[test]
fn case_lists_execute_before_and_after_idempotent_formatting() {
    let cwd = create_temp_dir("fmt-case-blocks");
    let path = cwd.join("case.fpas");
    write_text(
        &path,
        "program T; uses Std.Console; function Choose(): integer; begin case Some(2) of when Some(const V) if V > 0: const Answer: integer := V * 21; return Answer; when Some(const V): return 0; when None: return -1; end case; end function; begin WriteLn(Choose()); end.",
    );
    let run = ["run".into(), path.to_string_lossy().into_owned()];
    let (code, before, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before.replace("\r\n", "\n"), "42\n");
    let fmt = ["fmt".into(), path.to_string_lossy().into_owned()];
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    let formatted = fs::read_to_string(&path).expect("formatted source");
    assert_eq!(formatted.matches("begin").count(), 2, "{formatted}");
    let (code, after, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before, after);
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(fs::read_to_string(&path).expect("second format"), formatted);
    fs::remove_dir_all(cwd).expect("remove fixture");
}
