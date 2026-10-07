//! CLI validation and execution across AP13.6 expression formatting.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn legacy_expression_endings_and_extra_argument_semicolons_do_not_rewrite_source() {
    let cwd = create_temp_dir("fmt-expression-errors");
    let path = cwd.join("invalid.fpas");
    for source in [
        "program T; begin Apply(procedure() begin null; end); end.",
        "program T; begin Apply(P with X := 1; end); end.",
        "program T; begin Apply(procedure() begin null; end procedure;); end.",
    ] {
        write_text(&path, source);
        let (code, _, stderr) = run_cli_args_and_capture_output(
            &["fmt".into(), path.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_ne!(code, 0);
        assert!(stderr.contains("FP2001"), "{stderr}");
        assert_eq!(fs::read_to_string(&path).expect("source retained"), source);
    }
    fs::remove_dir_all(cwd).expect("remove fixture");
}

#[test]
fn expression_endings_execute_before_and_after_idempotent_formatting() {
    let cwd = create_temp_dir("fmt-expression-closers");
    let path = cwd.join("expressions.fpas");
    write_text(
        &path,
        "program T; uses Std.Console; type Point = record X: integer; end record; function Apply(F: function(P: Point): integer; P: Point): integer; begin return F(P); end function; begin const P: Point := record X := 1; end; WriteLn(Apply(function(P: Point): integer begin return P.X * 21; end function, P with X := 2; end with)); end.",
    );
    let run = ["run".into(), path.to_string_lossy().into_owned()];
    let (code, before, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before.replace("\r\n", "\n"), "42\n");
    let fmt = ["fmt".into(), path.to_string_lossy().into_owned()];
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    let formatted = fs::read_to_string(&path).expect("formatted source");
    let (code, after, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before, after);
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(fs::read_to_string(&path).expect("second format"), formatted);
    fs::remove_dir_all(cwd).expect("remove fixture");
}
