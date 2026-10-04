//! Equality constraints and resource capabilities survive compiled-unit interfaces.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::*;

const EQUALITY: &str = "unit Demo.Equality;\n    public function Same of (T: Equatable)(A: T; B: T): boolean;\n    begin return A = B; end function;\n    end unit;";

#[test]
fn imported_equatable_constraint_runs_with_collections() {
    let cwd = create_temp_dir("imported-equatable");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("equality.fpas"), EQUALITY);
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
        uses Demo.Equality as Equality; uses Std.Console as Console;
        begin
          Console.WriteLn(Equality.Same([[1], [2]], [[1], [2]]));
          Console.WriteLn(Equality.Same(['a': 1, 'b': 2], ['b': 2, 'a': 1]));
          Console.WriteLn(Equality.Same([1, 2], [2, 1]));
        end program;",
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (0, "true\ntrue\nfalse\n", "")
    );
}

#[test]
fn imported_equatable_rejects_nested_resources_with_a_located_diagnostic() {
    let cwd = create_temp_dir("imported-equatable-resource");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("equality.fpas"), EQUALITY);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Equality as Equality;
uses Std.Net as Net;
begin
const Value: option of (Net.Connection) := Option.None;
discard Equality.Same(Value, Value);
end program;"#,
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
    assert_eq!(records[0]["code"], "F2013");
    assert_eq!(records[0]["location"]["start"]["line"], 6);
}
