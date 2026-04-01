use super::*;

// ---------------------------------------------------------------------------
// go + Task.Wait
// ---------------------------------------------------------------------------

#[test]
fn go_and_task_wait() {
    let source = r#"program GoWait;
uses Std.Console, Std.Task;

function Worker(): integer;
begin
  return 42
end;

begin
  var T: task := go Worker();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn go_with_string_return() {
    let source = r#"program GoStr;
uses Std.Console, Std.Task;

function Greet(): string;
begin
  return 'hello'
end;

begin
  var T: task := go Greet();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_str.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "hello\n");
}

#[test]
fn go_with_arguments() {
    let source = r#"program GoArgs;
uses Std.Console, Std.Task;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var T: task := go Add(10, 32);
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_args.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

// ---------------------------------------------------------------------------
// Task.WaitAll
// ---------------------------------------------------------------------------

#[test]
fn task_wait_all() {
    let source = r#"program WaitAll;
uses Std.Console, Std.Task;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  var T1: task := go Double(1);
  var T2: task := go Double(2);
  var T3: task := go Double(3);
  var Results: array of task := [T1, T2, T3];
  Std.Task.WaitAll(Results);
  WriteLn('done')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("waitall.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "done\n");
}

#[test]
fn task_wait_all_empty_array() {
    let source = r#"program WaitAllEmpty;
uses Std.Console, Std.Task;

begin
  var Tasks: array of task := [];
  Std.Task.WaitAll(Tasks);
  WriteLn('ok')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("waitall_empty.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "ok\n");
}
