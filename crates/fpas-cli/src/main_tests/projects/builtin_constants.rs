//! Standard constant data and defaults survive cold and reused unit artifacts.

use super::*;

#[test]
fn builtin_constants_survive_imports_defaults_and_artifact_reuse() {
    let cwd = create_temp_dir("builtin-constant-values");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Constants;
        uses Std.Math as M; uses Std.Console as C;
        public const Pi: real := M.Pi;
        public const Values: array of (real) := [M.Pi];
        public const Same: boolean := [M.Pi] = [M.Pi];
        public const Mode: integer := C.Font8x8;
        public type Data = record public Value: real := M.Pi; end record;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
        uses Demo.Constants as Constants; uses Std.Math as M; uses Std.Console as C;
        begin
          const Skip := (Constants.Values = [M.Pi]) or (1 div 0 = 0);
          const Item := Constants.Data();
          C.WriteLn(Skip); C.WriteLn(Item.Value = M.Pi);
          C.WriteLn(Constants.Pi = M.Pi); C.WriteLn(Constants.Same);
          C.WriteLn(Constants.Mode = C.Font8x8);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "true\ntrue\ntrue\ntrue\ntrue\n", "")
        );
    }
    let args = ["build".to_owned(), project.to_string_lossy().into_owned()];
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("Reused program"), "{stdout}");
    assert!(cwd.join("model.fpascu").is_file());
    for guard in [
        "[M.Pi] = [M.Pi]",
        "Constants.Pi = M.Pi",
        "Constants.Values = [M.Pi]",
        "Constants.Same",
        "Constants.Mode = C.Font8x8",
        "Constants.Data() = Constants.Data(Value := M.Pi)",
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!(
                "program Main; uses Demo.Constants as Constants; uses Std.Math as M; uses Std.Console as C;\nbegin\n  const Value := ({guard}) and (1 div 0 = 0);\nend program;"
            ),
        );
        for command in ["check", "build", "run"] {
            let args =
                [command, "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
            assert_eq!(exit, 1, "{command}: {guard}: {stderr}");
            assert!(stdout.is_empty());
            let records = stderr
                .lines()
                .map(|line| {
                    serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic")
                })
                .collect::<Vec<_>>();
            assert_eq!(records.len(), 1, "{stderr}");
            assert_eq!(records[0]["code"], "F2020");
            assert_eq!(records[0]["phase"], "sema");
            assert_eq!(records[0]["location"]["start"]["line"], 3);
        }
    }
    fs::remove_dir_all(&cwd).expect("remove builtin constant project");
}
