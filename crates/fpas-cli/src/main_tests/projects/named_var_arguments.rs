//! Named reference arguments across compiled units and CLI diagnostics.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::*;
use crate::main_tests::support::run_source_and_capture_output;

#[test]
fn run_cli_named_var_arguments_reach_imported_storage_and_reuse_units() {
    let cwd = create_temp_dir("named-var-units");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/counter.fpas"),
        "unit App.Counter;
public var Total: integer := 10;
public procedure Add(var Value: integer; Step: integer);
begin Value := Value + Step; end procedure;
public procedure Forward(var Target: integer);
begin Add(Step := 3, Value := var Target); end procedure;
end unit;",
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;
uses App.Counter, Std.Console;
begin
  var Value: integer := 1;
  Add(Step := 2, Value := var Value);
  Forward(Target := var Value);
  Add(Value := var App.Counter.Total, Step := 4);
  WriteLn(Value); WriteLn(App.Counter.Total);
end.",
    );
    let mut sidecar_modified = None;
    for _ in 0..2 {
        let (code, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(stdout, "6\n14\n");
        assert!(stderr.is_empty(), "{stderr}");
        let modified = fs::metadata(cwd.join("src/counter.fpascu"))
            .expect("compiled unit")
            .modified()
            .expect("sidecar timestamp");
        if let Some(previous) = sidecar_modified {
            assert_eq!(modified, previous, "unchanged unit should be reused");
        }
        sidecar_modified = Some(modified);
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn run_cli_named_var_arguments_report_marker_alias_and_lifetime_errors() {
    for (body, code) in [
        ("Increase(Value := Counter);", "FP3027"),
        ("Swap(B := var Counter, A := var Counter);", "FP3029"),
        ("go Increase(Value := var Counter);", "FP3030"),
        (
            "const F: procedure(var Value: integer) := Increase; F(Value := var Counter);",
            "FP3026",
        ),
        ("Swap(A := var Counter, var Other);", "FP2016"),
    ] {
        let source = format!(
            "program Invalid;
uses Std.Tasks;
procedure Increase(var Value: integer); begin Value := Value + 1; end procedure;
procedure Swap(var A: integer; var B: integer); begin end procedure;
begin var Counter: integer := 0; var Other: integer := 0; {body} end."
        );
        let (exit, stdout, stderr) = run_source_and_capture_output("named-var.fpas", &source);
        assert_ne!(exit, 0, "{body}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(stderr.contains(code), "{body}: {stderr}");
    }
}
