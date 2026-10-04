//! Qualified immutable writes fail during compilation, before VM global guards.

use super::*;

const STATE: &str = r#"unit App.State;
public type Model = record public Values: array of (integer); end record;
public const Count: integer := 1;
public const Items: array of (integer) := [1];
public const Data: Model := Model(Values := [1]);
public const Mapping: dict of (string, array of (Model)) := ['key': [Model(Values := [1])]];
public  var Writable: Model := Model(Values := [1]);
end unit;"#;

#[test]
fn check_and_run_reject_secondary_unit_names_in_selected_imported_storage() {
    let cwd = create_temp_dir("imported-storage-alias-paths");
    let project = cwd.join("app.fpasprj");
    crate::test_support::write_program_fpasprj(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(&cwd.join("src/state.fpas"), STATE);
    for statement in [
        "var Value: integer := App.State.Writable.Values[0];",
        "App.State.Writable.Values[0] := 2;",
        "Arrays.Push(var App.State.Writable.Values, 2);",
    ] {
        write_text(
            &cwd.join("src/main.fpas"),
            &format!(
                "program Main;\nuses App.State as Store; uses Std.Arrays as Arrays;\nbegin\n  {statement}\nend program;"
            ),
        );
        for command in ["check", "run"] {
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
                &[
                    command.to_owned(),
                    "--diagnostics".to_owned(),
                    "json".to_owned(),
                    project.to_string_lossy().into_owned(),
                ],
                &cwd,
            );
            assert_eq!(exit, 1, "{statement}/{command}: {stderr}");
            assert!(stdout.is_empty(), "{stdout}");
            let diagnostics = stderr
                .lines()
                .map(|line| {
                    serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic")
                })
                .collect::<Vec<_>>();
            assert_eq!(diagnostics.len(), 1, "{stderr}");
            assert_eq!(diagnostics[0]["code"], "F2003", "{stderr}");
            assert_eq!(diagnostics[0]["location"]["start"]["line"], 4, "{stderr}");
        }
    }
    fs::remove_dir_all(&cwd).expect("remove test project");
}

#[test]
fn check_and_run_reject_immutable_imported_storage_with_located_sema_errors() {
    let cwd = create_temp_dir("immutable-import-diagnostics");
    let project = cwd.join("app.fpasprj");
    crate::test_support::write_program_fpasprj(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(&cwd.join("src/state.fpas"), STATE);
    let main = cwd.join("src/main.fpas");
    for target in [
        "Store.Count",
        "sToRe.Items[0]",
        "Store.Data.Values[0]",
        "Store.Mapping['key'][0].Values[0]",
    ] {
        write_text(
            &main,
            &format!(
                "program Main;\nuses App.State as Store;\nbegin\n  {target} := 2;\nend program;\n"
            ),
        );
        for command in ["check", "run"] {
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
                &[
                    command.to_string(),
                    "--diagnostics".to_string(),
                    "json".to_string(),
                    project.to_string_lossy().into_owned(),
                ],
                &cwd,
            );
            assert_eq!(exit, 1, "{target}/{command}: {stderr}");
            assert!(stdout.is_empty(), "{stdout}");
            let records = stderr
                .lines()
                .map(|line| {
                    serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic")
                })
                .collect::<Vec<_>>();
            assert_eq!(records.len(), 1, "{stderr}");
            let record = &records[0];
            assert_eq!(record["code"], "F2005", "{stderr}");
            assert_eq!(record["phase"], "sema", "{stderr}");
            assert_eq!(record["location"]["start"]["line"], 4, "{stderr}");
            assert_eq!(record["location"]["start"]["column"], 3, "{stderr}");
            assert!(
                record["source"]
                    .as_str()
                    .expect("source")
                    .ends_with("main.fpas")
            );
        }
    }
    fs::remove_dir_all(&cwd).expect("remove test project");
}

#[test]
fn mutable_imported_paths_and_value_snapshots_run_through_the_real_cli() {
    let cwd = create_temp_dir("immutable-import-snapshots");
    let project = cwd.join("app.fpasprj");
    crate::test_support::write_program_fpasprj(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(&cwd.join("src/state.fpas"), STATE);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.State as Store;
uses Std.Console as Console;
begin
   var Copy: dict of (string, array of (Store.Model)) := Store.Mapping;
  Copy['key'][0].Values[0] := 7;
  Store.Writable.Values[0] := Copy['key'][0].Values[0];
  Console.WriteLn(Store.Mapping['key'][0].Values[0]);
  Console.WriteLn(Copy['key'][0].Values[0]);
  Console.WriteLn(Store.Writable.Values[0]);
end program;"#,
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove test project");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "1\n7\n7\n");
    assert!(stderr.is_empty(), "{stderr}");
}
