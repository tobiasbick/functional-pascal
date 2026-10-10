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

const DECLARATIONS: &str = "
uses Std.Console, Std.Tasks;
type Reader = function(Value: integer): integer;
type Mutator = procedure(var Value: integer);
type Reporter = procedure(Output: channel of integer);
type Holder = record Readers: array of Reader; end record;
var Trace: integer := 0;
function Choose(): integer;
begin Trace := Trace * 10 + 1; return 0; end function;
function Argument(): integer;
begin Trace := Trace * 10 + 2; return 21; end function;
function Double(Value: integer): integer;
begin return Value * 2; end function;
procedure Increase(var Value: integer);
begin Value := Value + 1; end procedure;
procedure Report(Output: channel of integer);
begin discard Send(Output, 42); end procedure;
";

fn program(body: &str) -> String {
    format!(
        "program Indexed; {DECLARATIONS} begin
         const Readers: array of Reader := [Double];
         const Mutators: array of Mutator := [Increase];
         var Counter: integer := 0;
         {body} end."
    )
}

#[test]
fn indexed_callables_execute_selected_values_once_and_keep_reference_modes() {
    let source = program(
        "const Mapping: dict of string to Reader := ['double': Double];
         const Nested: array of array of Reader := [Readers];
         const Boxed: Holder := Holder(Readers := Readers);
         if Readers[Choose()](Argument()) <> 42 or Trace <> 12 then
           panic('callee and arguments must each run once, in order');
         end if;
         if Mapping['double'](21) <> 42 or Nested[0][0](21) <> 42
            or Boxed.Readers[0](21) <> 42 then panic('selected callable'); end if;
         Mutators[0](var Counter);
         if Counter <> 1 then panic('reference update'); end if;
         WriteLn('ok');",
    );
    let (exit, stdout, stderr) = cli("run", &source);
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "ok\n", ""));
}

#[test]
fn indexed_callables_work_as_retained_and_detached_tasks() {
    let source = program(
        "const Reports: array of Reporter := [Report];
         const Output: channel of integer := CreateChannel(1);
         const Job: task := go Readers[0](21);
         if Wait(Job) <> 42 then panic('task result'); end if;
         go Reports[0](Output);
         if ReceiveWithTimeout(Output, 1000).Unwrap() <> 42 then panic('detached'); end if;
         const ReportJob: task := go Reports[0](Output);
         Wait(ReportJob);
         if ReceiveWithTimeout(Output, 1000).Unwrap() <> 42 then panic('retained'); end if;
         WriteLn('ok');",
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
        ("discard Readers[0](Value := 1);", "FP3026", "named"),
        (
            "go Mutators[0](var Counter);",
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
          const Jobs: array of procedure() := [Work]; go Jobs[0]();",
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
