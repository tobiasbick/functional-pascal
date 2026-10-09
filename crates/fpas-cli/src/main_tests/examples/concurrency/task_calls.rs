//! End-to-end task-call fixtures for `docs/specs/grammar.ebnf` (`go_call`).
//!
//! **Documentation:** `docs/pascal/language/concurrency/go.md`.

use super::{repo_root, support};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

const DECLARATIONS: &str = "\
uses Std.Console, Std.Tasks;
type Worker = record
  Value: integer := 42;
  function Answer(Self: Worker): integer;
  begin
    return Self.Value;
  end function;
  function Next(Self: Worker): Worker;
  begin
    return Self;
  end function;
  function SendAnswer(Self: Worker; Output: channel of integer): integer;
  begin
    discard Send(Output, Self.Value);
    return Self.Value;
  end function;
end record;
type Group = record
  Items: array of Worker;
end record;
function Make(): Worker;
begin
  return Worker();
end function;
function MakeWorkers(): array of Worker;
begin
  return [Make()];
end function;
function MakeGroup(): Group;
begin
  return Group(Items := MakeWorkers());
end function;
function Direct(): integer;
begin
  return 42;
end function;
";

fn program(body: &str) -> String {
    format!("program TaskCalls;\n{DECLARATIONS}\nbegin\n{body}\nend.\n")
}

fn cli(command: &str, source: &str) -> (i32, String, String) {
    let dir = create_temp_dir("grammar-task-calls");
    let path = dir.join("task_calls.fpas");
    write_text(&path, source);
    let result = support::run_cli_args_and_capture_output(
        &[
            command.into(),
            "--std-lib".into(),
            repo_root().join("lib").to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove task-call fixture");
    result
}

#[test]
fn grammar_task_call_targets_pass_cli_check_and_return_results() {
    for target in [
        "Direct()",
        "Instance.Answer()",
        "Callable()",
        "Make().Answer()",
        "Make().Next().Answer()",
        "MakeGroup().Items[0].Answer()",
        "MakeWorkers()[0].Answer()",
        "(Make()).Answer()",
        "Worker(Value := 42).Answer()",
        "[Make()][0].Answer()",
        "(Make() with Value := 42; end with).Answer()",
    ] {
        let source = program(&format!(
            "const Instance: Worker := Make();\n\
             const Callable: function(): integer := Direct;\n\
             const Job: task := go {target};\n\
             const Answer: integer := Wait(Job);\nWriteLn(Answer);"
        ));
        let (exit, stdout, stderr) = cli("check", &source);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "", ""),
            "{target}"
        );
        let (exit, stdout, stderr) = cli("run", &source);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "42\n", ""),
            "{target}"
        );
    }
}

#[test]
fn grammar_detached_postfix_call_delivers_its_side_effect() {
    let source = program(
        "const Output: channel of integer := CreateChannel(1);\n\
         go Make().Next().SendAnswer(Output);\n\
         const Answer: integer := Receive(Output).Unwrap();\nWriteLn(Answer);",
    );
    let (exit, stdout, stderr) = cli("run", &source);
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "42\n", ""));
}

#[test]
fn grammar_non_call_task_targets_fail_cli_check_with_fp2005() {
    for target in [
        "Direct",
        "42",
        "Make().Value",
        "MakeWorkers()[0]",
        "Make().Answer() + 1",
        "(Make().Answer())",
    ] {
        let source = program(&format!("const Job: task := go {target};"));
        let (exit, stdout, stderr) = cli("check", &source);
        assert_eq!(exit, 1, "{target}\n{stderr}");
        assert!(stdout.is_empty(), "{target}\n{stdout}");
        assert!(stderr.contains("error[FP2005]"), "{target}\n{stderr}");
        assert!(stderr.contains("`go` requires"), "{target}\n{stderr}");
        assert_eq!(stderr.matches("error[").count(), 1, "{target}\n{stderr}");
    }
}

#[test]
fn grammar_call_shape_does_not_make_record_construction_a_task_target() {
    let source = program("const Job: task := go Worker();");
    let (exit, stdout, stderr) = cli("check", &source);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("error[FP3006]"), "{stderr}");
    assert!(stderr.contains("constructs a value"), "{stderr}");
    assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
}

#[test]
fn grammar_final_call_shape_does_not_make_a_value_field_callable() {
    let source = program("const Job: task := go Make().Value();");
    let (exit, stdout, stderr) = cli("check", &source);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("error[FP3006]"), "{stderr}");
    assert!(stderr.contains("not callable"), "{stderr}");
    assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
}
