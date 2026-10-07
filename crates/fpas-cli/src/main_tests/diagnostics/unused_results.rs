use super::*;

#[test]
fn unused_function_result_reports_code_location_and_result_fixes() {
    let source = "program T;\nfunction Value(): result of integer, string;\nbegin return Error('failed'); end function;\nbegin\n  Value();\nend.\n";
    let (exit, stderr) = support::run_and_capture_stderr("unused.fpas", source);
    assert_eq!(exit, 1);
    assert!(
        stderr.contains("unused.fpas:5:3: error[FP3022]: Unused function result"),
        "{stderr}"
    );
    assert!(
        stderr.contains("`case`")
            && stderr.contains("`try`")
            && stderr.contains("`discard Call();`"),
        "{stderr}"
    );
}

#[test]
fn unused_intrinsic_result_is_rejected_by_check_in_json_mode() {
    let cwd = create_temp_dir("unused-result-json");
    let main = cwd.join("main.fpas");
    write_text(&main, "program T; uses Std.Math; begin Abs(-1); end.");
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "check".into(),
            "--diagnostics".into(),
            "json".into(),
            main.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).unwrap();
    assert_eq!(exit, 1);
    assert!(stdout.is_empty());
    let diagnostic: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(diagnostic["code"], "FP3022");
    assert_eq!(diagnostic["phase"], "sema");
    assert_eq!(diagnostic["location"]["start"]["line"], 1);
    assert!(
        diagnostic["hint"]
            .as_str()
            .unwrap()
            .contains("`discard Call();`")
    );
}

#[test]
fn explicit_discard_runs_postfix_result_once_and_procedure_calls_still_run() {
    let source = "program T; uses Std.Console;
      var Count: integer := 0;
      type Box = record Value: integer;
        function Get(Self: Box): result of integer, string;
        begin Count := Count + 1; return Error('ignored'); end function;
        procedure Show(Self: Box); begin WriteLn(Self.Value); end procedure;
      end record;
      function Make(): Box;
      begin Count := Count + 1; return record Value := 7; end; end function;
      begin discard Make().Get(); Make().Show(); WriteLn(Count); end.";
    let (exit, stdout, stderr) = support::run_source_and_capture_output("discard.fpas", source);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "7\n3\n");
}

#[test]
fn imported_function_result_is_rejected_then_explicit_discard_runs() {
    let cwd = create_temp_dir("imported-unused-result");
    let unit = cwd.join("Values.fpas");
    write_text(
        &unit,
        "unit Values;
      public function Get(): Option of integer; begin return Some(7); end function;
      end unit;",
    );
    let main = cwd.join("main.fpas");
    write_text(&main, "program T; uses Values; begin Values.Get(); end.");
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), cwd.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(exit, 1);
    assert!(stderr.contains("FP3022"), "{stderr}");
    write_text(
        &main,
        "program T; uses Values; begin discard Values.Get(); end.",
    );
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), cwd.to_string_lossy().into_owned()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).unwrap();
    assert_eq!(exit, 0, "{stderr}");
}
