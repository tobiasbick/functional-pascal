//! Source var modes survive unit linking and persisted program reuse.

use super::*;

const STORAGE: &str = r#"unit Demo.Storage;
uses Std.Arrays as Arrays;
public type Box = record public Items: array of (integer); end record;
public type Action = procedure(var Value: integer);
public  var Data: Box := Box(Items := [1, 2]);
public procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
public function Select(): Action; begin return Increase; end function;
public procedure Append(var Items: array of (integer)); begin Arrays.Push(var Items, 7); end procedure;
public function Replace of (T)(var Value: T; NewValue: T): T; begin const Before: T := Value; Value := NewValue; return Before; end function;
end unit;"#;

#[test]
fn caller_references_survive_compiled_units_program_encoding_and_warm_reuse() {
    let cwd = create_temp_dir("compiled-caller-references");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("storage.fpas"), STORAGE);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Storage as Store; uses Std.Console as Console; uses Std.Arrays as Arrays;
procedure Forward(var Value: integer); begin Store.Increase(var Value); end procedure;
begin
  const Before: Store.Box := Store.Data;
  Forward(var Store.Data.Items[0]);
  Store.Select()(var Store.Data.Items[1]);
   var Count: integer := 40;
  const Action: Store.Action := Store.Select();
  Action(var Count);
  Console.WriteLn(Count);
  Store.Append(var Store.Data.Items);
  Console.WriteLn(Arrays.Pop(var Store.Data.Items));
  Console.WriteLn(Store.Data.Items[0]);
  Console.WriteLn(Store.Data.Items[1]);
  Console.WriteLn(Before.Items[0]);
  Console.WriteLn(Store.Replace(var Count, 42));
  Console.WriteLn(Count);
end program;"#,
    );
    let args = [
        String::from("build"),
        project.to_string_lossy().into_owned(),
    ];
    let cold = support::run_cli_args_and_capture_output(&args, &cwd);
    let warm = support::run_cli_args_and_capture_output(&args, &cwd);
    let result = support::run_cli_and_capture_output(&project, &cwd);
    let sidecar_exists = cwd.join("storage.fpascu").is_file();
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(cold.0, 0, "{}", cold.2);
    assert_eq!(warm.0, 0, "{}", warm.2);
    assert!(warm.1.contains("Reused program"), "{}", warm.1);
    assert!(sidecar_exists);
    assert_eq!(result.0, 0, "{}", result.2);
    assert_eq!(result.1, "41\n7\n2\n3\n1\n41\n42\n");
}

#[test]
fn caller_references_reject_the_same_imported_root_through_mixed_case_aliases() {
    let cwd = create_temp_dir("duplicate-qualified-var-root");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("storage.fpas"), STORAGE);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Storage as Store;
procedure Both(var A: integer; var B: integer); begin null; end procedure;
begin Both(var Store.Data.Items[0], var store.Data.Items[1]); end program;"#,
    );
    for command in ["check", "run"] {
        let args =
            [command, "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{command}: {stderr}");
        assert!(stdout.is_empty());
        let diagnostics: Vec<serde_json::Value> = stderr
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON diagnostic"))
            .collect();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["code"] == "F2006"
                    && diagnostic["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("root"))),
            "{stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}
