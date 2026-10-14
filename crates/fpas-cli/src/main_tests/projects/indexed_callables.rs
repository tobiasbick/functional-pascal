//! Indexed function and procedure values through CLI checking and execution.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`.

use super::*;

fn cli(command: &str, source: &str) -> (i32, String, String) {
    let directory = create_temp_dir("indexed-callables");
    let path = directory.join("main.fpas");
    write_text(&path, source);
    let result = support::run_cli_args_and_capture_output(
        &[command.to_owned(), path.to_string_lossy().into_owned()],
        &directory,
    );
    fs::remove_dir_all(directory).expect("remove indexed-call fixture");
    result
}

const DECLARATIONS: &str = "\nuses Std.Console as Console, Std.Tasks as Tasks;\ntype Reader = function(Value: integer): integer;\ntype Mutator = procedure(var Value: integer);\ntype Reporter = procedure(Output: channel of integer);\ntype Holder = record Readers: array of Reader; end record;\nvar Trace: integer := 0;\nfunction Choose(): integer;\nbegin Trace := Trace * 10 + 1; return 0; end function;\nfunction Argument(): integer;\nbegin Trace := Trace * 10 + 2; return 21; end function;\nfunction Double(Value: integer): integer;\nbegin return Value * 2; end function;\nprocedure Increase(var Value: integer);\nbegin Value := Value + 1; end procedure;\nprocedure Report(Output: channel of integer);\nbegin discard Tasks.Send(Output, 42); end procedure;\n";

fn program(body: &str) -> String {
    format!(
        "program Indexed; {DECLARATIONS} begin scope
         const Readers: array of Reader := [Double];
         const Mutators: array of Mutator := [Increase];
         var Counter: integer := 0;
         {body} end scope; end."
    )
}

#[test]
fn indexed_callables_execute_selected_values_once_and_keep_reference_modes() {
    let source = program(
        "const Mapping: dict of string to Reader := ['double': Double];\n         const Nested: array of array of Reader := [Readers];\n         const Boxed: Holder := Holder(Readers := Readers);\n         if Readers[Choose()](Argument()) <> 42 or Trace <> 12 then\n           panic('callee and arguments must each run once, in order');\n         end if;\n         if Mapping['double'](21) <> 42 or Nested[0][0](21) <> 42\n            or Boxed.Readers[0](21) <> 42 then panic('selected callable'); end if;\n         Mutators[0](var Counter);\n         if Counter <> 1 then panic('reference update'); end if;\n         Console.WriteLn('ok');",
    );
    let (exit, stdout, stderr) = cli("run", &source);
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "ok\n", ""));
}

#[test]
fn indexed_callables_work_with_bound_and_discarded_handles() {
    let source = program(
        "const Reports: array of Reporter := [Report];\n         const Output: channel of integer := Tasks.CreateChannel(1);\n         const Job: task of integer := go Readers[0](21);\n         if Tasks.Wait(Job) <> 42 then panic('task result'); end if;\n         discard go Reports[0](Output);\n         if Tasks.ReceiveWithTimeout(Output, 1000).Unwrap() <> 42 then panic('detached'); end if;\n         const ReportJob: task := go Reports[0](Output);\n         Tasks.Wait(ReportJob);\n         if Tasks.ReceiveWithTimeout(Output, 1000).Unwrap() <> 42 then panic('retained'); end if;\n         Console.WriteLn('ok');",
    );
    let (exit, stdout, stderr) = cli("run", &source);
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "ok\n", ""));
}

#[test]
fn indexed_callables_reject_invalid_arguments_and_task_escapes() {
    for (body, code, hint) in [
        (
            "discard Readers['bad'](1);",
            "FP3006",
            "Array index must be integer",
        ),
        ("discard Readers[0]('bad');", "FP3006", "Type mismatch"),
        ("Mutators[0](Counter);", "FP3027", "var"),
        (
            "discard Readers[0](Unknown := 1);",
            "FP3025",
            "Expected parameter names: Value",
        ),
        (
            "discard go Mutators[0](var Counter);",
            "FP3030",
            "cannot pass a `var` argument",
        ),
        (
            "const Job: task := go Mutators[0](var Counter);",
            "FP3030",
            "cannot pass a `var` argument",
        ),
        (
            "const Values: array of integer := [1]; discard Values[0]();",
            "FP3006",
            "not callable",
        ),
        (
            "const Work: procedure() := procedure() begin Counter := Counter + 1; end procedure;
          const Jobs: array of procedure() := [Work]; discard go Jobs[0]();",
            "FP3016",
            "task-bound",
        ),
        (
            "const Work: procedure() := procedure() begin Counter := Counter + 1; end procedure;
          const Jobs: array of procedure() := [Work]; const Job: task := go Jobs[0]();",
            "FP3016",
            "task-bound",
        ),
    ] {
        let (exit, stdout, stderr) = cli("check", &program(body));
        assert_eq!(exit, 1, "{body}\n{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{body}\n{stderr}"
        );
        assert!(stderr.contains(hint), "{body}\n{stderr}");
        assert_eq!(stderr.matches("error[").count(), 1, "{body}\n{stderr}");
    }
}
