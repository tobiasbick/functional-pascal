//! Computed initialization, inference and immutable storage across unit reuse.

use super::*;

const MODEL: &str = r#"unit Demo.Bindings;
public type Box = record public Values: array of (integer); end record;
var Calls: integer := 0;
function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
public const First: integer := Next();
public const Second: integer := Next();
public const StaticValues: array of (integer) := [1, 2];
public const Computed: Box := Box(Values := [Next()]);
public function Count(): integer; begin return Calls; end function;
end unit;"#;

#[test]
fn computed_and_aggregate_consts_initialize_once_after_cold_and_warm_builds() {
    let cwd = create_temp_dir("binding-initializer-reuse");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("bindings.fpas"), MODEL);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Bindings as Model; uses Std.Console as Console;
begin
  var Copy := Model.StaticValues;
  const Snapshot := Copy;
  Copy[0] := 9;
  const Value := if true then Model.Computed else Model.Box(Values := [0]) end if;
  Console.WriteLn(Model.First);
  Console.WriteLn(Model.Second);
  Console.WriteLn(Value.Values[0]);
  Console.WriteLn(Model.Count());
  Console.WriteLn(Model.StaticValues[0]);
  Console.WriteLn(Snapshot[0]);
  Console.WriteLn(Copy[0]);
end program;"#,
    );
    let args = ["build".to_owned(), project.to_string_lossy().into_owned()];
    let cold = support::run_cli_args_and_capture_output(&args, &cwd);
    let warm = support::run_cli_args_and_capture_output(&args, &cwd);
    let sidecar_exists = cwd.join("bindings.fpascu").is_file();
    let runs = (0..2)
        .map(|_| support::run_cli_and_capture_output(&project, &cwd))
        .collect::<Vec<_>>();
    fs::remove_dir_all(&cwd).expect("remove binding project");
    assert_eq!(cold.0, 0, "{}", cold.2);
    assert_eq!(warm.0, 0, "{}", warm.2);
    assert!(warm.1.contains("Reused program"), "{}", warm.1);
    assert!(sidecar_exists);
    for (exit, stdout, stderr) in runs {
        assert_eq!(exit, 0, "{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert_eq!(stdout, "1\n2\n3\n3\n1\n1\n9\n");
    }
}

#[test]
fn imported_consts_stay_readonly_and_computed_labels_remain_non_static() {
    let cwd = create_temp_dir("binding-negative-imports");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("bindings.fpas"), MODEL);
    for (body, code) in [
        ("Model.StaticValues[0] := 3;", "F2005"),
        ("Model.Computed.Values[0] := 3;", "F2005"),
        (
            "const Copy := Model.Computed; Copy.Values[0] := 3;",
            "F2005",
        ),
        (
            "case 1 of when Model.First: null; else null; end case;",
            "F2014",
        ),
        ("const Missing := [];", "F2006"),
        ("var Missing := [:]; Missing['key'] := 1;", "F2006"),
        ("const Empty := Option.None;", "F2006"),
        ("const Value := if true then 1 else 'one' end if;", "F2006"),
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!("program Main; uses Demo.Bindings as Model; begin {body} end program;"),
        );
        for command in ["check", "run"] {
            let args =
                [command, "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
            assert_eq!(exit, 1, "{command}/{body}: {stderr}");
            assert!(stdout.is_empty(), "{stdout}");
            let records = stderr
                .lines()
                .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("diagnostic"))
                .collect::<Vec<_>>();
            assert!(
                records.iter().any(|record| record["code"] == code),
                "{stderr}"
            );
            assert!(!stderr.contains("F9001"), "{stderr}");
        }
    }
    fs::remove_dir_all(&cwd).expect("remove binding project");
}
