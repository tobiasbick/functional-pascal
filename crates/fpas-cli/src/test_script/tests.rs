//! Unit tests for test script parsing and application.

use super::{apply_script_to_vm, load_script, parse_script_text, sidecar_path_for_test};
use fpas_compiler::compile;
use fpas_parser::parse;
use std::path::Path;

#[test]
fn sidecar_path_replaces_fpas_extension() {
    let path = Path::new("tests/console/readln_test.fpas");
    assert_eq!(
        sidecar_path_for_test(path),
        Path::new("tests/console/readln_test.script.toml")
    );
}

#[test]
fn parse_readln_event() {
    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "Alice"
"#,
        Path::new("demo.script.toml"),
    )
    .expect("parse");
    assert_eq!(script.events.len(), 1);
}

#[test]
fn parse_unknown_event_type_is_error() {
    let err = parse_script_text(
        r#"
[[event]]
type = "teleport"
"#,
        Path::new("bad.script.toml"),
    )
    .unwrap_err();
    assert!(err.contains("Unknown event type `teleport`"));
}

#[test]
fn parse_console_event_types_are_rejected() {
    let err = parse_script_text(
        r#"
[[event]]
type = "console_key"
kind = "Escape"
"#,
        Path::new("bad.script.toml"),
    )
    .unwrap_err();
    assert!(err.contains("Unknown event type `console_key`"));
}

#[test]
fn apply_readln_script_runs_readln_test_program() {
    let source = "\
program T;
uses Std.Console, Std.Test;
begin
  AssertTrue(ReadLn() = 'Alice')
end.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_vm::Vm::new(executable);

    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "Alice"
"#,
        Path::new("readln.script.toml"),
    )
    .expect("parse");
    apply_script_to_vm(&mut vm, &script).expect("apply");
    vm.run().expect("run");
}

#[test]
fn graph_event_without_headless_flag_is_rejected_on_apply() {
    let script = parse_script_text(
        r#"
[[event]]
type = "graph_key"
kind = "Escape"
"#,
        Path::new("graph.script.toml"),
    )
    .expect("parse");
    let source = "program T; begin end.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_vm::Vm::new(executable);
    let err = apply_script_to_vm(&mut vm, &script).unwrap_err();
    assert!(err.contains("headless_graph"));
}

#[test]
fn apply_graph_escape_script_runs_headless_graph_test_program() {
    let source = "\
program T;
uses Std.Console, Std.Graph, Std.Test;

mutable var QuitSeen: boolean := false;

procedure OnPaint(App: Application);
begin
  Application.Clear(App, $00102030);
  Application.Present(App)
end;

function OnKeyPressed(App: Application; Key: KeyEvent): boolean;
begin
  if Key.kind = KeyKind.Escape then
  begin
    QuitSeen := true;
    Application.HostRequestQuit(App);
    return true
  end;
  return false
end;

begin
  var App: Application := Application.Open(32, 24, 'Graph smoke');
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnKeyPressed := Some(OnKeyPressed);
  end;
  Application.Configure(App, Handlers);
  Application.Run(App);
  AssertTrue(QuitSeen)
end.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_vm::Vm::new(executable);

    let script = parse_script_text(
        r#"
[config]
headless_graph = true

[[event]]
type = "graph_key"
kind = "Escape"
"#,
        Path::new("graph_smoke.script.toml"),
    )
    .expect("parse");
    apply_script_to_vm(&mut vm, &script).expect("apply");
    fpas_std::with_headless_graph_backend_for_tests(|| vm.run()).expect("run");
}

#[test]
fn apply_readln_events_are_consumed_in_script_order() {
    let source = "\
program T;
uses Std.Console, Std.Test;
begin
  AssertEquals(ReadLn(), 'first');
  AssertEquals(ReadLn(), 'second');
  AssertEquals(ReadLn(), 'third')
end.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_vm::Vm::new(executable);

    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "first"

[[event]]
type = "readln"
line = "second"

[[event]]
type = "readln"
line = "third"
"#,
        Path::new("order.script.toml"),
    )
    .expect("parse");
    apply_script_to_vm(&mut vm, &script).expect("apply");
    vm.run().expect("run");
}

#[test]
fn load_script_reads_file_from_disk() {
    let dir = crate::test_support::create_temp_dir("fpas-script-load");
    let path = dir.join("input.script.toml");
    crate::test_support::write_text(&path, "[[event]]\ntype = \"readln\"\nline = \"ok\"\n");
    let script = load_script(&path).expect("load");
    assert_eq!(script.events.len(), 1);
}
