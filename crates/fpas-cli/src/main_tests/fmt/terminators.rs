//! CLI coverage for required terminators and shared control-body endings.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn missing_final_terminator_is_rejected_without_rewriting_the_file() {
    let cwd = create_temp_dir("fmt-missing-terminator");
    let path = cwd.join("missing.fpas");
    let source = "program P; uses Std.Console; begin WriteLn('x') end.";
    write_text(&path, source);
    let (code, _, stderr) =
        run_cli_args_and_capture_output(&["fmt".into(), path.to_string_lossy().into_owned()], &cwd);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("FP2001") && stderr.contains("Expected `;` after statement"),
        "{stderr}"
    );
    assert_eq!(
        fs::read_to_string(path).expect("source is preserved"),
        source
    );
    fs::remove_dir_all(cwd).expect("remove fixture");
}

#[test]
fn terminated_branches_and_loops_keep_their_execution_and_format_idempotently() {
    let cwd = create_temp_dir("fmt-terminated-control");
    let path = cwd.join("control.fpas");
    write_text(
        &path,
        "program P; uses Std.Console; begin var X: integer := 0; if true then WriteLn('then'); else WriteLn('else'); end if; while X < 2 do X := X + 1; end while; repeat X := X - 1; until X = 0; WriteLn(X); end.",
    );
    let args = ["run".into(), path.to_string_lossy().into_owned()];
    let (code, before, stderr) = run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before.replace("\r\n", "\n"), "then\n0\n");
    let fmt_args = ["fmt".into(), path.to_string_lossy().into_owned()];
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt_args, &cwd);
    assert_eq!(code, 0, "{stderr}");
    let formatted = fs::read_to_string(&path).expect("formatted source");
    let (code, after, stderr) = run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before, after);
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt_args, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        fs::read_to_string(path).expect("second formatting"),
        formatted
    );
    fs::remove_dir_all(cwd).expect("remove fixture");
}
