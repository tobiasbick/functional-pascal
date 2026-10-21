//! Task admission checks the callable, implicit receiver, and every explicit argument.

use super::{SEMA_TASK_BOUND_CALLABLE, check_errors, check_ok};

#[test]
fn go_rejects_known_local_arguments_in_each_value_wrapper() {
    for (parameter, argument) in [
        ("procedure()", "Work"),
        ("array of procedure()", "[Work]"),
        ("option of procedure()", "Some(Work)"),
        ("result of (procedure(), string)", "Ok(Work)"),
        ("dict of string to procedure()", "['work': Work]"),
        ("WorkBox", "WorkBox(Work := Work)"),
    ] {
        let source = format!(
            r#"program T;
type WorkBox = record Work: procedure(); end record;
procedure Consume(Value: {parameter}); begin end procedure;
begin scope
  var Count: integer := 0;
  const Work := procedure() begin Count := Count + 1; end procedure;
  discard go Consume({argument});
end scope; end."#
        );
        let errors = check_errors(&source);
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
                .count(),
            1,
            "{argument}: {errors:#?}"
        );
    }
}

#[test]
fn go_rejects_local_receivers_even_when_the_method_result_is_scalar() {
    for receiver in ["Boxed", "(Boxed)"] {
        let errors = check_errors(&format!(
            r#"program T;
type WorkBox = record
  Work: procedure();
  function Number(Self: WorkBox): integer; begin return 7; end function;
end record;
begin scope
  var Count: integer := 0;
  const Work := procedure() begin Count := Count + 1; end procedure;
  const Boxed := WorkBox(Work := Work);
  discard go {receiver}.Number();
end scope; end."#
        ));
        assert_eq!(
            errors
                .iter()
                .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
                .count(),
            1,
            "{receiver}: {errors:#?}"
        );
    }
}

#[test]
fn scalar_arguments_and_local_selection_callbacks_remain_legal() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;
type WorkBox = record Work: procedure(); Safe: integer; end record;
procedure Consume(Value: integer); begin end procedure;
begin scope
  var Count: integer := 0;
  const Work := procedure() begin Count := Count + 1; end procedure;
  const Boxed := WorkBox(Work := Work, Safe := 7);
  discard go Consume(Boxed.Safe);
  discard go Consume((Boxed).Safe);
  const Timer := Tasks.TimerCase(0, Work);
  discard Tasks.CloseWaitCase(Timer);
end scope; end."#,
    );
}

#[test]
fn unknown_arguments_keep_the_runtime_guard_and_immutable_arguments_are_valid() {
    check_ok(
        r#"program T;
procedure Consume(Value: procedure()); begin Value(); end procedure;
procedure Forward(Value: procedure());
begin scope discard go Consume(Value); end scope; end procedure;
begin scope
  const Work := procedure() begin end procedure;
  discard go Consume(Work);
end scope; end."#,
    );
}
