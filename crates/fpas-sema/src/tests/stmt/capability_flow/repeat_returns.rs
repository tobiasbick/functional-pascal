use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_NON_BOOLEAN_CONDITION;

fn program(exit: &str, condition_type: &str) -> String {
    format!(
        "program RepeatReturn;
        function Safe(): result of (procedure(), procedure());
        begin
          var Count: integer := 0;
          const Local: procedure() := procedure()
            begin Count := Count + 1; end procedure;
          const Outcome: result of ({condition_type}, procedure()) := Error(Local);
          repeat
            {exit}
          until try Outcome;
          return Ok(procedure() begin end procedure);
        end function;
        begin scope discard go Safe(); end scope; end."
    )
}

#[test]
fn unreachable_repeat_condition_does_not_publish_a_try_error_return() {
    for exit in ["return Ok(procedure() begin end procedure);", "break;"] {
        check_ok(&program(exit, "boolean"));
    }
}

#[test]
fn unreachable_repeat_condition_still_requires_boolean_type() {
    for exit in ["return Ok(procedure() begin end procedure);", "break;"] {
        let errors = check_errors(&program(exit, "integer"));
        assert_eq!(errors.len(), 1, "{exit}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_NON_BOOLEAN_CONDITION, "{exit}");
    }
}
