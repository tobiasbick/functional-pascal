use super::{assert_succeeds, run_program};

#[test]
fn discard_evaluates_once_without_unwrapping_result_or_invoking_callable() {
    assert_succeeds(
        "program T;
      var Calls: integer := 0;
      function Produce(): result of integer, string;
      begin Calls := Calls + 1; return Error('ignored'); end function;
      begin discard Produce();
      discard function(): integer begin Calls := Calls + 10; return 1; end function;
      if Calls <> 1 then panic('discard evaluated incorrectly'); end if;
      end.",
    );
}

#[test]
fn discarded_panics_still_propagate() {
    assert!(
        run_program(
            "program T;
      function Fail(): integer; begin panic('expected failure'); end function;
      begin discard Fail(); end."
        )
        .is_err()
    );
}

#[test]
fn unused_discard_results_do_not_remove_runtime_failures() {
    for operand in ["1 div 0", "[1][2]"] {
        assert!(
            run_program(&format!("program T; begin discard {operand}; end.")).is_err(),
            "{operand}"
        );
    }
}

#[test]
fn discard_preserves_postfix_evaluation_and_closure_captures() {
    assert_succeeds(
        "program T;
      type Box = record Value: integer;
        function GetValue(Self: Box): integer; begin return Self.Value; end function;
      end record;
      function Make(): Box; begin return Box( Value := 42 ); end function;
      begin discard Make().GetValue();
      const Value: integer := 1;
      discard function(): integer begin return Value; end function;
      end.",
    );
}
