//! Execution coverage for constraints propagated through generic calls.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::super::assert_succeeds;

#[test]
fn generic_forwarding_executes_with_integer_and_real_values() {
    assert_succeeds(
        "program Main;\n         function Twice of (T: Numeric)(Value: T): T;\n         begin return Value + Value; end function;\n         function Forward of (U: Numeric)(Value: U): U;\n         begin return Twice(Value); end function;\n         function Again of (V: Numeric)(Value: V): V;\n         begin return Forward(Value); end function;\n         begin\n           if Again(21) <> 42 then panic('integer forwarding'); end if;\n           if Again(1.25) <> 2.5 then panic('real forwarding'); end if;\n         end program;",
    );
}

#[test]
fn generic_forwarding_executes_nested_array_callback_and_procedure_calls() {
    assert_succeeds(
        "program Main;\n         function Double(Value: integer): integer;\n         begin return Value + Value; end function;\n         function Apply of (T: Numeric)(Values: array of (T); Action: function(Item: T): T): T;\n         begin return Action(Values[0]); end function;\n         function Forward of (U: Numeric)(Values: array of (U); Action: function(Input: U): U): U;\n         begin return Apply(Values, Action); end function;\n         procedure Consume of (T: Comparable)(Value: T);\n         begin if Value <> Value then panic('procedure forwarding'); end if; end procedure;\n         procedure Pass of (U: Numeric)(Value: U);\n         begin Consume(Value); end procedure;\n         begin\n           if Forward([21], Double) <> 42 then panic('callback forwarding'); end if;\n           Pass(42);\n         end program;",
    );
}
