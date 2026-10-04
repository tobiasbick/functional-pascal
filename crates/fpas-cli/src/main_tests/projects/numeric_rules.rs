//! Numeric diagnostics, imported static values and runner isolation.

use super::*;

#[test]
fn aggregate_static_failures_and_exported_guards_use_checked_values() {
    let cwd = create_temp_dir("numeric-static-aggregates");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("model.fpas"), "unit Demo.Numbers;
        public const Same: boolean := [1] = [1];
        public const Different: boolean := [1] = [2];
        public const Values: array of (integer) := [1];
        public const Mapping: dict of (string, integer) := ['a': 1, 'b': 2];
        public type Data = record public Value: integer; end record;
        public type Alias = Data;
        public const Item: Data := Data(Value := 1);
        public const Items: array of (Data) := [Item];
        public const Maybe: Option of (integer) := Option.Some(1);
        public const Outcome: Result of (integer, string) := Result.Ok(1);
        public type Choice = enum Empty; Present(Value: integer); end enum;
        public const Empty: Choice := Choice.Empty;
        public type Settings = record public Value: boolean := ([1] = [2]) and (1 div 0 = 0); end record;
        end unit;");
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
        uses Demo.Numbers as Numbers; uses Std.Console as Console;
        begin const No := Numbers.Different and (1 div 0 = 0);
          const Yes := Numbers.Same or (1 mod 0 = 0);
          const ArraySkip := (Numbers.Values = [1]) or (1 div 0 = 0);
          const DictSkip := (Numbers.Mapping = ['b': 2, 'a': 1]) or (1 div 0 = 0);
          const RecordSkip := (Numbers.Item = Numbers.Alias(Value := 1)) or (1 div 0 = 0);
          const NestedSkip := (Numbers.Items = [Numbers.Data(Value := 1)]) or (1 div 0 = 0);
          const OptionSkip := (Numbers.Maybe = Option.Some(1)) or (1 div 0 = 0);
          const ResultSkip := (Numbers.Outcome = Result.Ok(1)) or (1 div 0 = 0);
          const EnumSkip := (Numbers.Empty = Numbers.Choice.Empty) or (1 div 0 = 0);
          const Item := Numbers.Settings();
          Console.WriteLn(No); Console.WriteLn(Yes); Console.WriteLn(Item.Value);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "false\ntrue\nfalse\n", "")
        );
    }
    for expression in [
        "[1 div 0] = [0]",
        "[0] = [1 div 0]",
        "0 in [1 mod 0]",
        "([1] = [1]) and (1 div 0 = 0)",
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!("program Main;\nbegin\n  const Value := {expression};\nend program;"),
        );
        for command in ["check", "build", "run"] {
            let args =
                [command, "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
            assert_eq!(exit, 1, "{command}: {expression}: {stderr}");
            assert!(stdout.is_empty());
            let records = diagnostic_records(&stderr);
            assert_eq!(records.len(), 1, "{stderr}");
            assert_eq!(records[0]["code"], "F2020");
            assert_eq!(records[0]["phase"], "sema");
            assert_eq!(records[0]["location"]["start"]["line"], 3);
            assert!(
                records[0]["source"]
                    .as_str()
                    .unwrap()
                    .ends_with("main.fpas")
            );
        }
    }
    let model = cwd.join("model.fpas");
    let original = fs::read_to_string(&model).expect("aggregate model source");
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Numbers as Numbers; begin const Value := (Numbers.Values = [1]) or (1 div 0 = 0); end program;",
    );
    let (exit, _, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    assert_eq!(exit, 0, "{stderr}");
    write_text(
        &model,
        &original.replace(
            "Values: array of (integer) := [1]",
            "Values: array of (integer) := [2]",
        ),
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 1, "changed aggregate metadata: {stderr}");
    assert_eq!(diagnostic_records(&stderr)[0]["code"], "F2020");
    write_text(&model, &original);
    for guard in [
        "Numbers.Values = [1]",
        "1 in Numbers.Values",
        "Numbers.Mapping = ['b': 2, 'a': 1]",
        "Numbers.Item.Value = 1",
        "Numbers.Maybe = Option.Some(1)",
        "Numbers.Empty = Numbers.Choice.Empty",
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!(
                "program Main; uses Demo.Numbers as Numbers; begin const Value := ({guard}) and (1 div 0 = 0); end program;"
            ),
        );
        let args =
            ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{guard}: {stderr}");
        assert_eq!(
            diagnostic_records(&stderr)[0]["code"],
            "F2020",
            "{guard}: {stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("remove aggregate numeric project");
}

#[test]
fn imported_ieee_constants_and_record_defaults_survive_artifact_reuse() {
    let cwd = create_temp_dir("numeric-unit-reuse");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Numbers;
        public const Maximum: integer := 9223372036854775807;
        public const Infinite: real := 1 / 0;
        public const Nan: real := 0 / 0;
        public type Settings = record public Value: real := 1 / -0.0; end record;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Numbers as Numbers; uses Std.Console as Console;
        begin const Value := Numbers.Settings();
          Console.WriteLn(Numbers.Maximum = 9223372036854775807);
          Console.WriteLn(Numbers.Infinite > 1.0e300);
          Console.WriteLn(Numbers.Nan <> Numbers.Nan);
          Console.WriteLn(Numbers.Nan <= Numbers.Infinite);
          Console.WriteLn(Value.Value < -1.0e300);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "true\ntrue\ntrue\nfalse\ntrue\n", "")
        );
    }
    let args = ["build".to_owned(), project.to_string_lossy().into_owned()];
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("Reused program"), "{stdout}");
    assert!(cwd.join("model.fpascu").is_file());
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Numbers as Numbers; begin const Value := Numbers.Maximum + 1; end program;",
    );
    for command in ["check", "build", "run"] {
        let args =
            [command, "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{command}: {stderr}");
        assert!(stdout.is_empty());
        let records = diagnostic_records(&stderr);
        assert_eq!(records.len(), 1, "{stderr}");
        assert_eq!(records[0]["code"], "F2020");
        assert_eq!(records[0]["phase"], "sema");
    }
    fs::remove_dir_all(&cwd).expect("remove numeric project");
}

#[test]
fn invalid_unit_static_operations_keep_their_original_source() {
    let cwd = create_temp_dir("numeric-unit-errors");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Numbers as Numbers; begin null; end program;",
    );
    for declaration in [
        "public const Value: integer := 1 div 0;",
        "public type Data = record public Value: integer := 9223372036854775807 + 1; end record;",
    ] {
        write_text(
            &cwd.join("model.fpas"),
            &format!("unit Demo.Numbers;\n{declaration}\nend unit;"),
        );
        let args = ["run", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{stderr}");
        assert!(stdout.is_empty());
        let records = diagnostic_records(&stderr);
        assert_eq!(records.len(), 1, "{stderr}");
        assert_eq!(records[0]["code"], "F2020");
        assert!(
            records[0]["source"]
                .as_str()
                .unwrap()
                .ends_with("model.fpas")
        );
        assert_eq!(records[0]["location"]["start"]["line"], 2);
    }
    fs::remove_dir_all(&cwd).expect("remove numeric project");
}

#[test]
fn cli_and_test_runner_distinguish_static_errors_from_runtime_panics() {
    let cwd = create_temp_dir("numeric-runner");
    for (name, body, code, phase, run_exit) in [
        (
            "static",
            "const Value := 9223372036854775807 + 1;",
            "F2020",
            "sema",
            1,
        ),
        (
            "overflow",
            "var Value := 9223372036854775807 + 1;",
            "F4012",
            "runtime",
            2,
        ),
        ("division", "var Value := 1 div 0;", "F4001", "runtime", 2),
        ("modulo", "var Value := 1 mod 0;", "F4002", "runtime", 2),
        (
            "conversion",
            "const Value := Math.Trunc(0 / 0);",
            "F4012",
            "runtime",
            2,
        ),
        (
            "shift",
            "const Value := Bits.ShiftRight(0, 64);",
            "F4012",
            "runtime",
            2,
        ),
    ] {
        let path = cwd.join(format!("{name}_test.fpas"));
        write_text(
            &path,
            &format!(
                "program Main; uses Std.Math as Math; uses Std.Bits as Bits;\nbegin\n  {body}\nend program;"
            ),
        );
        let args = ["run", "--diagnostics", "json", &path.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, run_exit, "{name}: {stderr}");
        assert!(stdout.is_empty());
        let records = diagnostic_records(&stderr);
        assert_eq!(records.len(), 1, "{stderr}");
        assert_eq!(records[0]["code"], code);
        assert_eq!(records[0]["phase"], phase);
        assert_eq!(records[0]["location"]["start"]["line"], 3);
    }
    write_text(
        &cwd.join("pass_test.fpas"),
        "program Pass; uses Std.Test as Test; begin const Nan := 0 / 0; Test.AssertFalse(Nan = Nan); end program;",
    );
    for jobs in ["1", "2"] {
        let args = [
            "test",
            "--diagnostics",
            "json",
            "--report",
            "json",
            "--jobs",
            jobs,
            &cwd.to_string_lossy(),
        ]
        .map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        let report: serde_json::Value = serde_json::from_str(&stdout).expect("test report");
        assert_eq!(
            exit, 2,
            "compile-error fixtures determine the runner exit: {stderr}"
        );
        assert_eq!(report["summary"]["failed"], 0);
        assert_eq!(report["summary"]["runtime_errors"], 5);
        assert_eq!(report["summary"]["compile_errors"], 1);
        assert_eq!(report["summary"]["passed"], 1);
        let mut codes = diagnostic_records(&stderr)
            .iter()
            .map(|record| record["code"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        codes.sort();
        assert_eq!(
            codes,
            ["F2020", "F4001", "F4002", "F4012", "F4012", "F4012"]
        );
    }
    fs::remove_dir_all(&cwd).expect("remove numeric runner fixtures");
}

fn diagnostic_records(stderr: &str) -> Vec<serde_json::Value> {
    stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON diagnostic"))
        .collect()
}
