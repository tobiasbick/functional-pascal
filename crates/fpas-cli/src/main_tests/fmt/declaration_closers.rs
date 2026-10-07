//! CLI rejection and execution regressions for named declaration endings.

use super::{create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;

#[test]
fn legacy_declaration_ending_is_rejected_without_rewriting_source() {
    let cwd = create_temp_dir("fmt-legacy-declaration-ending");
    let path = cwd.join("legacy.fpas");
    let source = "unit Demo; function Answer(): integer; begin return 42; end; end unit;";
    write_text(&path, source);
    let (code, _, stderr) =
        run_cli_args_and_capture_output(&["fmt".into(), path.to_string_lossy().into_owned()], &cwd);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("FP2001") && stderr.contains("end function;"),
        "{stderr}"
    );
    assert_eq!(
        fs::read_to_string(&path).expect("source is preserved"),
        source
    );
    fs::remove_dir_all(cwd).expect("remove fixture");
}

#[test]
fn named_declarations_keep_execution_and_format_idempotently() {
    let cwd = create_temp_dir("fmt-named-declarations");
    let path = cwd.join("declarations.fpas");
    let unit_path = cwd.join("empty.fpas");
    write_text(&unit_path, "unit Empty; end unit;");
    write_text(&path, "program Demo; uses Std.Console;
        type Status = enum Ready; end enum;
        type Point = record Value: integer;
            function ReadValue(Self: Point): integer; begin return Self.Value; end function;
        end record;
        function Answer(): integer;
            procedure Prepare(); begin return; end procedure;
        begin Prepare(); const P: Point := record Value := 42; end; return P.ReadValue(); end function;
        procedure Show(); begin WriteLn(Answer()); end procedure;
        begin Show(); end.");
    let run = ["run".into(), path.to_string_lossy().into_owned()];
    let (code, before, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before.replace("\r\n", "\n"), "42\n");
    let fmt = [
        "fmt".into(),
        path.to_string_lossy().into_owned(),
        unit_path.to_string_lossy().into_owned(),
    ];
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    let formatted = fs::read_to_string(&path).expect("formatted source");
    let (code, after, stderr) = run_cli_args_and_capture_output(&run, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(before, after);
    let (code, _, stderr) = run_cli_args_and_capture_output(&fmt, &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        fs::read_to_string(path).expect("second formatting"),
        formatted
    );
    assert_eq!(
        fs::read_to_string(unit_path).expect("formatted unit"),
        "unit Empty;\n\nend unit;\n"
    );
    fs::remove_dir_all(cwd).expect("remove fixture");
}
