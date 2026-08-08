use super::*;

#[test]
fn retained_task_spawn_and_wait_match_stack_execution() {
    let execution = assert_both_succeed(
        "\
program RegisterTasks;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var T: task := go Add(20, 22);
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn detached_task_executes_on_register_pool() {
    assert_both_succeed(
        "\
program RegisterDetached;

procedure Work();
begin
  Std.Console.WriteLn('worker')
end;

begin
  go Work()
end.",
    );
}

#[test]
fn timeslice_preserves_nested_frames_and_live_aggregate_registers() {
    assert_both_succeed(
        "\
program RegisterTaskFrames;

function Burn(Count: integer): integer;
begin
  mutable var I: integer := 0;
  while I < Count do
    I := I + 1;
  return I
end;

function Work(): integer;
begin
  var Values: array of integer := [40, 2];
  return Burn(700) - 700 + Values[0] + Values[1]
end;

begin
  var T: task := go Work();
  if Std.Task.Wait(T) <> 42 then panic('task state was not restored')
end.",
    );
}

#[test]
fn cooperative_sleep_releases_register_pool_worker() {
    assert_both_succeed(
        "\
program RegisterTaskSleep;

function Work(Value: integer): integer;
begin
  Std.Time.Sleep(1);
  return Value
end;

begin
  var A: task := go Work(42);
  Std.Task.Wait(A)
end.",
    );
}

#[test]
fn wait_all_keeps_register_task_results_available() {
    assert_both_succeed(
        "\
program RegisterWaitAll;

function Work(Value: integer): integer;
begin
  return Value
end;

begin
  var A: task := go Work(20);
  var B: task := go Work(22);
  Std.Task.WaitAll([A, B]);
  Std.Task.Wait(A);
  Std.Task.Wait(B)
end.",
    );
}

#[test]
fn mutable_capture_cannot_cross_register_task_boundary() {
    let source = "\
program RegisterTaskBound;

function Make(): function(): integer;
begin
  mutable var Value: integer := 41;
  return function(): integer
  begin
    Value := Value + 1;
    return Value
  end
end;

begin
  var Work: function(): integer := Make();
  var T: task := go Work();
  Std.Task.Wait(T)
end.";
    let old = run_old(source).expect_err("stack path must reject task-bound closure");
    let register = run_register(source).expect_err("register path must reject task-bound closure");
    assert_eq!(register.code, old.code);
    assert!(register.message.contains("task-bound"));
}
