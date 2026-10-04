//! Reexported defaults execute through independently compiled units and retained visibility.
//!
//! **Documentation:** `docs/pascal/language/types/type-aliases.md`.

use super::*;
use std::path::{Path, PathBuf};

#[test]
fn obsolete_record_hint_uses_the_original_imported_alias_identity() {
    let cwd = create_temp_dir("obsolete-imported-record");
    let project = write_alias_project(&cwd);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main; uses Demo.Second as Api; begin const Value: Api.Settings := record Required := 1; end record; end program;"#,
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let errors = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{stderr}");
    assert_eq!(errors[0]["code"], "F2006");
    assert!(
        errors[0]["hint"]
            .as_str()
            .is_some_and(|hint| hint.contains("demo.model.settings")),
        "{stderr}"
    );
    assert_eq!(errors[0]["location"]["start"]["line"], 1);
    assert!(!stderr.contains("F9001"));
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

fn write_alias_project(cwd: &Path) -> PathBuf {
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model; const Base: integer := 1 + 2;
         public type Settings = record
           public Required: integer;
           public Count: integer := Base;
         end record;
         public type LocalAlias = Settings;
         end unit;",
    );
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;\n         public type Settings = Model.LocalAlias;\n         public type SettingsList = array of (Settings);\n         public function CountOf(Value: Settings): integer;\n         begin return Value.Count; end function;\n         end unit;",
    );
    write_text(
        &cwd.join("second.fpas"),
        "unit Demo.Second; uses Demo.Facade as Facade;
         public type Settings = Facade.Settings;
         public type SettingsList = Facade.SettingsList;
         end unit;",
    );
    project
}

#[test]
fn alias_chains_and_collection_aliases_execute_defaults_and_reuse_sidecars() {
    let cwd = create_temp_dir("record-default-alias-chain");
    let project = write_alias_project(&cwd);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main; uses Demo.Second as Second; uses Demo.Facade as Facade;
         uses Std.Console as Console;
         begin
           const Value: Second.Settings := Second.Settings(Required := 1);
           Console.WriteLn(Value.Count);
           const Items: Second.SettingsList := [Second.Settings(Required := 2)];
           Console.WriteLn(Items[0].Count);
           Console.WriteLn(Facade.CountOf(Facade.Settings(Required := 3)));
           const Override: Second.Settings := Second.Settings(Required := 4, Count := 42);
           Console.WriteLn(Override.Count);
         end program;"#,
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "3\n3\n3\n42\n", "")
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}

#[test]
fn alias_preserves_the_required_field_diagnostic() {
    let cwd = create_temp_dir("record-default-alias-required");
    let project = write_alias_project(&cwd);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Second as Second;
begin
const Value: Second.Settings := Second.Settings();
end program;"#,
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{stderr}");
    assert_eq!(records[0]["code"], "F2015");
    assert_eq!(records[0]["location"]["start"]["line"], 4);
    assert!(records[0]["message"].as_str().unwrap().contains("Required"));
}

#[test]
fn alias_defaults_do_not_allow_private_record_construction() {
    let cwd = create_temp_dir("record-default-alias-private");
    let project = write_alias_project(&cwd);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
         public type Settings = record
           public Required: integer := 1;
           Count: integer := 3;
         end record;
         public type LocalAlias = Settings;
         end unit;",
    );
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;\n         public type Settings = Model.LocalAlias;\n         public type SettingsList = array of (Settings);\n         end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Second as Second;
begin
const Value: Second.Settings := Second.Settings();
end program;"#,
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{stderr}");
    assert_eq!(records[0]["code"], "F2017");
    assert_eq!(records[0]["location"]["start"]["line"], 4);
}
