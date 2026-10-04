//! Nonliteral defaults through source-adjacent compiled units and CLI diagnostics.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::*;

#[test]
fn exported_record_default_expressions_execute_and_reuse_compiled_units() {
    let cwd = create_temp_dir("record-default-expressions");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
         const Base: integer := 1 + 2;
         public type Settings = record
           public Count: integer := Base * 2;
           public Scale: real := Base / 2;
           public Label: string := 'de' + 'fault';
           public Enabled: boolean := not false and (Base > 2);
         end record;
         end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Model as Model; uses Std.Console as Console;\n         begin\n           var Value: Model.Settings := Model.Settings();\n           Console.WriteLn(Value.Count); Console.WriteLn(Value.Scale);\n           Console.WriteLn(Value.Label); Console.WriteLn(Value.Enabled);\n           var Override: Model.Settings := Model.Settings(Count := 42);\n           Console.WriteLn(Override.Count);\n         end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "6\n1.5\ndefault\ntrue\n42\n", "")
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}

#[test]
fn invalid_exported_record_default_has_a_source_diagnostic() {
    let cwd = create_temp_dir("record-default-type-error");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;\npublic type Settings = record\npublic Count: integer := 'wrong' + 'type';\nend record;\nend unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Model as Model; begin null; end program;",
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{stderr}");
    assert_eq!(records[0]["code"], "F2006");
    assert_eq!(records[0]["location"]["start"]["line"], 3);
}
