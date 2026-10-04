use super::super::{assert_succeeds, run_program};

#[test]
fn computed_callable_targets_spawn_with_explicit_arguments() {
    assert_succeeds(
        r#"program ComputedTasks;

uses Std.Tasks as Tasks;

type Handler = function(Value: integer): integer;

function Make(Base: integer): Handler;
begin
  return function(Value: integer): integer begin
    return Base + Value;
  end function;
end function;

begin
  const Returned: task := go Make(40)(2);
  const Functions: array of (Handler) := [Make(30)];
  const Indexed: task := go Functions[0](12);
  const Parenthesized: task := go (Functions[0])(12);
  if Tasks.Wait(Returned) <> 42 then
    panic('returned task target');
  end if;

  if Tasks.Wait(Indexed) <> 42 then
    panic('indexed task target');
  end if;

  if Tasks.Wait(Parenthesized) <> 42 then
    panic('parenthesized task target');
  end if;
  const Messages: channel of (integer) := Tasks.CreateChannel(1);
  const Actions: array of (procedure(Value: integer)) := [procedure(Value: integer) begin
    discard Tasks.Send(Messages, Value);
  end procedure];
  const ProcedureTask: task := go Actions[0](42);
  Tasks.Wait(ProcedureTask);
  case Tasks.Receive(Messages) of
    when Result.Ok(const Value):
      if Value <> 42 then
        panic('procedure task argument');
      end if;
    when Result.Error(const Message):
      panic(Message);
  end case;
end program;
"#,
    );
}

#[test]
fn computed_targets_retain_dynamic_task_bound_protection() {
    for call in ["Make()()", "[Make()][0]()", "(Make())()"] {
        let source = format!(
            r#"
program ComputedTaskBound;
function Make(): function(): integer;
begin
   var Count: integer := 0;
  return function(): integer begin Count := Count + 1; return Count; end function;
end function;
begin const Handle: task := go {call}; end program;
"#
        );
        let error = run_program(&source).expect_err("task-bound target must be rejected");
        assert!(error.message.contains("task-bound"), "{call}: {error:?}");
    }
}
