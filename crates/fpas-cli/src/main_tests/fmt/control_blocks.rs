//! CLI rejection and execution checks for AP13.4 control statements.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn legacy_control_syntax_is_rejected_without_rewriting_source() {
    let cwd = create_temp_dir("fmt-legacy-control");
    let path = cwd.join("legacy.fpas");
    let source = "program T; begin if true then WriteLn('old'); end.";
    write_text(&path, source);
    let (code, _, stderr) =
        run_cli_args_and_capture_output(&["fmt".into(), path.to_string_lossy().into_owned()], &cwd);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("FP2001") && stderr.contains("end if;"),
        "{stderr}"
    );
    assert_eq!(fs::read_to_string(&path).expect("source retained"), source);
    fs::remove_dir_all(cwd).expect("remove fixture");
}

#[test]
fn control_blocks_execute_before_and_after_idempotent_formatting() {
    let cwd = create_temp_dir("fmt-control-blocks");
    let path = cwd.join("control.fpas");
    write_text(&path, "program T; uses Std.Console;
        function Choose(): integer; begin if false then return 0; elsif true then return 42; else return 1; end if; end function;
        begin for I: integer := 1 to 1 do while true do WriteLn(Choose()); break; end while; end for; end.");
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
