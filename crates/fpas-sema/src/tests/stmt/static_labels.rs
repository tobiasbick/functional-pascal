//! Static scalar labels and endpoints share the recursive pattern rules.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;

#[test]
fn scalar_labels_reject_variables_calls_and_each_dynamic_range_endpoint() {
    for label in ["Lower", "Next()", "Lower..3", "1..Upper", "Next()..Next()"] {
        let errors = check_errors(&format!(
            "program Main;
            var Lower: integer := 1; var Upper: integer := 3;
            function Next(): integer; begin return 1; end function;
            begin case 2 of when {label}: null; else null; end case; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
            "{label}: {errors:#?}"
        );
    }
}

#[test]
fn static_operators_and_constants_match_while_computed_values_belong_in_guards() {
    check_ok(
        "program Main;
        const Lower: integer := 1 + 1; const Upper: integer := Lower + 3;
        function Next(): integer; begin return 2; end function;
        begin case 2 of when Lower..Upper: null; else null; end case;
          case 2 of when const Value if Value = Next(): null; else null; end case;
        end program;",
    );
}

#[test]
fn try_and_decision_values_are_rejected_in_scalar_and_nested_value_patterns() {
    for label in ["(try Next())", "(if true then 1 else 2 end if)"] {
        let errors = check_errors(&format!(
            "program Main;\n            function Next(): Result of (integer, string); begin return Result.Ok(1); end function;\n            function Probe(): Result of (integer, string); begin\n              case 1 of when {label}: return Result.Ok(1); else return Result.Ok(0); end case;\n            end function; begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
            "{errors:#?}"
        );
    }
    let errors = check_errors(
        "program Main;
        type Choice = enum Present(Value: integer); Missing; end enum;
        function Next(): integer; begin return 1; end function;
        begin var Value: Choice := Choice.Present(1);
          case Value of when Choice.Present((Next())): null;
            when Choice.Present(_): null; when Choice.Missing: null; end case;
        end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
        "{errors:#?}"
    );
}
