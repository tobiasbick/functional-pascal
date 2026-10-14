//! End-to-end task-call fixtures for `docs/specs/grammar.ebnf` (`go_call`).
//!
//! **Documentation:** `docs/pascal/language/concurrency/go.md`.

use super::{repo_root, support};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

const DECLARATIONS: &str = "uses Std.Console as Console, Std.Tasks as Tasks;\ntype Worker = record\n  Value: integer := 42;\n  function Answer(Self: Worker): integer;\n  begin\n    return Self.Value;\n  end function;\n  function Next(Self: Worker): Worker;\n  begin\n    return Self;\n  end function;\n  function SendAnswer(Self: Worker; Output: channel of integer): integer;\n  begin\n    discard Tasks.Send(Output, Self.Value);\n    return Self.Value;\n  end function;\n  procedure Report(Self: Worker; Output: channel of integer);\n  begin\n    discard Tasks.Send(Output, Self.Value);\n  end procedure;\n  function Increment(Self: Worker; var Count: integer): integer;\n  begin\n    Count := Count + Self.Value;\n    return Count;\n  end function;\nend record;\ntype Group = record\n  Items: array of Worker;\nend record;\nfunction Make(): Worker;\nbegin\n  return Worker();\nend function;\nfunction MakeWorkers(): array of Worker;\nbegin\n  return [Make()];\nend function;\nfunction MakeGroup(): Group;\nbegin\n  return Group(Items := MakeWorkers());\nend function;\nfunction Direct(): integer;\nbegin\n  return 42;\nend function;\n";

fn program(body: &str) -> String {
    format!("program TaskCalls;\n{DECLARATIONS}\nbegin scope\n{body}\nend scope; end.\n")
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
            "const Instance: Worker := Make();\nconst Callable: function(): integer := Direct;\nconst Job: task of integer := go {target};\nconst Answer: integer := Tasks.Wait(Job);\nConsole.WriteLn(Answer);"
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
fn grammar_discarded_postfix_call_delivers_its_side_effect() {
    let source = program(
        "const Output: channel of integer := Tasks.CreateChannel(1);\ndiscard go Make().Next().SendAnswer(Output);\nconst Answer: integer := Tasks.Receive(Output).Unwrap();\nConsole.WriteLn(Answer);",
    );
    let (exit, stdout, stderr) = cli("run", &source);
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "42\n", ""));
}

#[test]
fn grammar_postfix_procedure_tasks_deliver_their_side_effects() {
    for spawn in [
        "discard go Make().Report(Output);",
        "const Job: task := go Make().Report(Output); Tasks.Wait(Job);",
    ] {
        let source = program(&format!(
            "const Output: channel of integer := Tasks.CreateChannel(1);\n{spawn}\nConsole.WriteLn(Tasks.ReceiveWithTimeout(Output, 1000).Unwrap());"
        ));
        let (exit, stdout, stderr) = cli("run", &source);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "42\n", ""),
            "{spawn}"
        );
    }
}

#[test]
fn grammar_postfix_tasks_reject_writable_arguments() {
    for spawn in [
        "discard go Make().Increment(var Count);",
        "const Job: task of integer := go Make().Increment(var Count);",
    ] {
        let source = program(&format!("var Count: integer := 0; {spawn}"));
        let (exit, stdout, stderr) = cli("check", &source);
        assert_eq!(exit, 1, "{spawn}\n{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(stderr.contains("error[FP3030]"), "{spawn}\n{stderr}");
        assert!(stderr.contains("cannot pass a `var` argument"), "{stderr}");
        assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
    }
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
        let source = program(&format!("const Job: task of integer := go {target};"));
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
