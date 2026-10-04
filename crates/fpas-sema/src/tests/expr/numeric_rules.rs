//! Static failures, checked integer endpoints and IEEE real arithmetic.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_INVALID_STATIC_OPERATION;

#[test]
fn static_integer_failures_have_an_operator_diagnostic() {
    for expression in [
        "9223372036854775807 + 1",
        "(-9223372036854775807 - 1) - 1",
        "9223372036854775807 * 2",
        "-(-9223372036854775807 - 1)",
        "(-9223372036854775807 - 1) div -1",
        "(-9223372036854775807 - 1) mod -1",
        "1 div 0",
        "1 mod 0",
    ] {
        for source in [
            format!("program T; const Value: integer := {expression}; begin null; end program;"),
            format!("program T; begin const Value := {expression}; end program;"),
            format!(
                "program T; begin case 1 of when {expression}: null; else null; end case; end program;"
            ),
            format!(
                "program T; begin case 1 of when 0..({expression}): null; else null; end case; end program;"
            ),
            format!(
                "program T; type Data = record Value: integer := {expression}; end record; begin null; end program;"
            ),
        ] {
            let errors = check_errors(&source);
            assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
            assert_eq!(
                errors[0].code, SEMA_INVALID_STATIC_OPERATION,
                "{source}: {errors:#?}"
            );
            assert!(errors[0].help.is_some(), "{errors:#?}");
        }
    }
}

#[test]
fn static_aggregate_children_and_lexical_constants_are_checked() {
    for expression in [
        "[1 div 0]",
        "['key': 1 mod 0]",
        "Option.Some(9223372036854775807 + 1)",
        "Data(Value := 1 div 0)",
        "Data(Value := 1) with Value := 1 div 0; end with",
    ] {
        let errors = check_errors(&format!(
            "program T; type Data = record Value: integer; end record; begin const Value := {expression}; end program;"
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
    }
    let errors = check_errors(
        "program T; const Limit: integer := 9223372036854775807; begin const Value := Limit + 1; end program;",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
    check_ok(
        "program T; const Limit: integer := 9223372036854775807; begin begin const Limit := 0; const Value := Limit + 1; end; end program;",
    );
}

#[test]
fn short_circuiting_skips_invalid_static_operations() {
    check_ok(
        "program T; const No: boolean := false and (1 div 0 = 0); const Yes: boolean := true or (1 mod 0 = 0); begin const Nested := [No, Yes]; end program;",
    );
    for expression in [
        "true and (1 div 0 = 0)",
        "false or (1 div 0 = 0)",
        "false xor (1 div 0 = 0)",
    ] {
        let errors = check_errors(&format!(
            "program T; begin const Value := {expression}; end program;"
        ));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
    }
}

#[test]
fn runtime_expressions_remain_runtime_operations() {
    check_ok(
        "program T; function Evaluate(Value: integer): integer; begin return Value + 1; end function; begin var Value := 9223372036854775807 + 1; const Computed := Evaluate(9223372036854775807); end program;",
    );
}

#[test]
fn static_real_division_and_integer_endpoints_are_valid() {
    check_ok(
        "program T; const Minimum: integer := -9223372036854775807 - 1; const Maximum: integer := 9223372036854775807; const Infinite: real := 1 / 0; const Nan: real := 0.0 / 0.0; const Ordered: boolean := Nan < Infinite; begin null; end program;",
    );
}

#[test]
fn aggregate_comparisons_and_membership_check_all_reached_components() {
    for expression in [
        "[1 div 0] = [0]",
        "[0] <> [1 mod 0]",
        "[[9223372036854775807 + 1]] = [[0]]",
        "['key': 1 div 0] = ['key': 0]",
        "1 in [1, 1 div 0]",
        "0 in [1 div 0: false]",
        "Option.Some(1 div 0) = Option.Some(0)",
        "Result.Ok(1 div 0) = Result.Ok(0)",
        "Data(Value := 1 div 0) = Data(Value := 0)",
        "(Data(Value := 0) with Value := 1 div 0; end with) = Data(Value := 0)",
    ] {
        let errors = check_errors(&format!(
            "program T; type Data = record Value: integer; end record; begin const Value := {expression}; end program;"
        ));
        assert_eq!(errors.len(), 1, "{expression}: {errors:#?}");
        assert_eq!(
            errors[0].code, SEMA_INVALID_STATIC_OPERATION,
            "{expression}: {errors:#?}"
        );
    }
}

#[test]
fn aggregate_guards_preserve_lazy_evaluation_and_dictionary_normalization() {
    for (guard, expected) in [
        ("[1] = [1]", true),
        ("[1] <> [1]", false),
        ("Choice.First = Choice.First", true),
        ("Choice.First = Choice.Second", false),
        ("[Payload.Empty] = [Payload.Empty]", true),
        ("[0.0 / 0.0] = [0.0 / 0.0]", false),
        ("1 in [2, 1]", true),
        ("1 in [2, 3]", false),
        ("'a' in ['a': 1, 'b': 2]", true),
        ("['a': 0, 'a': 1, 'b': 2] = ['b': 2, 'a': 1]", true),
        ("Option.Some([1]) = Option.Some([1])", true),
        ("Result.Ok([1]) = Result.Ok([2])", false),
        ("Data(Value := 1) = Alias(Value := 1)", true),
        ("Data() = Data(Value := 1)", true),
        (
            "(Data() with Value := 2; end with) = Data(Value := 1)",
            false,
        ),
    ] {
        for operation in ["and", "or"] {
            let source = format!(
                "program T; type Choice = enum First; Second; end enum; type Payload = enum Empty; Present(Value: integer); end enum; type Data = record Value: integer := 1; end record; type Alias = Data; begin const Value := ({guard}) {operation} (1 div 0 = 0); end program;"
            );
            if (operation == "and") == expected {
                let errors = check_errors(&source);
                assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
                assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
            } else {
                check_ok(&source);
            }
        }
    }
}

#[test]
fn static_aggregate_binding_values_respect_lexical_identity() {
    check_ok(
        "program T; const Values: array of (integer) := [1]; begin
        begin const Values := [2]; const Skip := (Values = [1]) and (1 div 0 = 0); end;
        const Skip := (Values = [1]) or (1 div 0 = 0);
        end program;",
    );
    let errors = check_errors("program T; type Data = record Value: integer := 1; end record;
        begin const Item := Data(); const Value := (Item.Value = 1) and (1 div 0 = 0); end program;");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
}
