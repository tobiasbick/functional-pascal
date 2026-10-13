//! Integer restrictions, conversions, static bounds, and integration with existing type rules.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_NON_CONSTANT_EXPRESSION, SEMA_NON_EXHAUSTIVE_CASE, SEMA_TYPE_MISMATCH,
    SEMA_WRONG_ARGUMENT_COUNT,
};

const TYPES: &str =
    "program T; type Percent = 0..100; type Same = (0)..(10 * 10); type Small = 0..10;";

#[test]
fn subrange_bounds_conversions_identity_and_scalar_widening_are_checked() {
    check_ok(
        "program T; const Min: integer := -2; const Max: integer := Min + 4;
        type Signed = Min..Max; type Single = 1..1; type Copy = -2..2; type Alias = Signed;
        function Echo(Value: integer): integer; begin return Value; end function;
        function Read(): Signed; begin return Signed(Echo(1)); end function;
        begin const Low := Signed(-2); const High := Signed(2); const One := Single(1);
        const A: Copy := Low; const B: Alias := A; const Raw: integer := B;
        const Sum: integer := Low + High; const Negated: integer := -Low;
        const Divided: real := High / One; const Widened: integer := Echo(High);
        const Wrapped: Signed := Read(); const Other: Single := Single(High div 2);
        end.",
    );
}

#[test]
fn subranges_reject_invalid_bounds_and_static_violations() {
    for body in [
        "type R = 10..0;",
        "type R = 0.0..100;",
        "type R = true..false;",
        "type R = 'a'..'z';",
        "type E = enum A; B; end enum; type R = E.A..E.B;",
    ] {
        let errors = check_errors(&format!("program T; {body} begin end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{body}: {errors:?}"
        );
    }
    for initializer in [
        "Percent(1.0)",
        "Percent(true)",
        "Percent('1')",
        "Percent(Value := 1)",
        "Percent(var Raw)",
    ] {
        let errors = check_errors(&format!(
            "{TYPES} begin var Raw: integer := 1; const Bad := {initializer}; end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{initializer}: {errors:?}"
        );
    }
    for initializer in ["Percent()", "Percent(1, 2)"] {
        let errors = check_errors(&format!("{TYPES} begin const Bad := {initializer}; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_WRONG_ARGUMENT_COUNT),
            "{initializer}: {errors:?}"
        );
    }
    for initializer in [
        "Percent(-1)",
        "Percent(101)",
        "Percent(50 + 51)",
        "Small(Percent(11))",
    ] {
        let errors = check_errors(&format!("{TYPES} begin const Bad := {initializer}; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("outside")),
            "{initializer}: {errors:?}"
        );
    }
    for bounds in ["Read()..100", "Computed..100", "0..Computed"] {
        let errors = check_errors(&format!(
            "program T; function Read(): integer; begin return 1; end function; const Computed: integer := Read(); type R = {bounds}; begin end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
            "{bounds}: {errors:?}"
        );
    }
}

#[test]
fn subrange_static_record_and_conditional_bounds_keep_value_declaration_order() {
    check_ok(
        "program T; const DefaultLimit: integer := 100;
        type Settings = record Lower: integer; Upper: integer := DefaultLimit; end record;
        const Limits: Settings := Settings(Lower := -10);
        const Minimum: integer := -9223372036854775807 - 1;
        type Full = Minimum..9223372036854775807;
        type Fields = Limits.Lower..Limits.Upper;
        type Conditional = if true then -10 else 0 end if..case 1 of when 1: 100; else 50; end case;
        const Before: Future := Future(1); const Size: integer := 10; type Future = 0..Size;
        begin const Low := Full(Minimum); const High := Full(9223372036854775807);
        const Same: Fields := Conditional(50); end.",
    );
    let errors =
        check_errors("program T; type R = 0..Later; const Later: integer := 100; begin end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_CONSTANT_EXPRESSION),
        "{errors:?}"
    );
}

#[test]
fn subranges_require_explicit_narrowing_in_all_value_positions() {
    for body in [
        "var P: Percent := 1;",
        "var P := Percent(1); P := 2;",
        "const P: Percent := Small(1);",
        "const P: Small := Percent(1);",
        "const P: Percent := Percent(1) + Percent(2);",
        "Take(1);",
    ] {
        let source = format!(
            "{TYPES} procedure Take(Value: Percent); begin end procedure; begin {body} end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("checked conversion"))),
            "{body}: {errors:?}"
        );
    }
    let errors = check_errors(&format!(
        "{TYPES} function Bad(): Percent; begin return 1; end function; begin end."
    ));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("Percent(Value)"))),
        "{errors:?}"
    );
}

#[test]
fn subrange_var_storage_and_container_types_remain_exact() {
    check_ok(&format!("{TYPES} procedure Set(var Value: Percent); begin Value := Percent(5); end procedure;
        begin var P := Same(1); Set(var P); const Values: array of Percent := [Same(1), Percent(2)]; end."));
    check_ok(&format!("{TYPES} begin const Index := Small(1);
        var Values: array of integer := [0, 1]; Values[Index] := Percent(5);
        const At: integer := Values[Index]; const Text: string := 'ab'; const Letter: string := Text[Index];
        const Lookup: dict of integer to string := [1: 'one']; const Found: string := Lookup[Index];
        const InArray: boolean := Index in Values; const InDict: boolean := Index in Lookup;
        const Typed: dict of Percent to string := [Percent(1): 'one']; const SameKey: string := Typed[Same(1)]; end."));
    for body in [
        "var P := Percent(1); SetInteger(var P);",
        "var P: integer := 1; SetPercent(var P);",
        "var P := Small(1); SetPercent(var P);",
        "const Values: array of integer := [Percent(1)];",
        "const Values: array of Percent := [1];",
        "var Values: array of Percent := [Percent(1)]; Values[0] := 1;",
        "const Callback: function(Value: integer): integer := Echo;",
        "const Values: array of Percent := [Percent(1)]; const Found: boolean := 1 in Values;",
        "const Lookup: dict of Percent to string := [Percent(1): 'one']; const Found: string := Lookup[1];",
    ] {
        let errors = check_errors(&format!("{TYPES} procedure SetInteger(var Value: integer); begin Value := 200; end procedure;
            procedure SetPercent(var Value: Percent); begin Value := Percent(5); end procedure;
            function Echo(Value: Percent): Percent; begin return Value; end function; begin {body} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{body}: {errors:?}"
        );
    }
}

#[test]
fn subranges_have_comparison_and_printing_constraints_without_numeric_closure() {
    check_ok(&format!("{TYPES} function Max<T: Comparable>(A: T; B: T): T; begin if A > B then return A; else return B; end if; end function;
        procedure Show<T: Printable>(Value: T); begin end procedure;
        begin const P: Percent := Max(Percent(1), Same(2)); Show(P);
        const Equal: boolean := P = 2; const Earlier: boolean := P < Small(3);
        const Index: dict of Percent to string := [Percent(1): 'one']; end."));
    let errors = check_errors(&format!(
        "{TYPES} function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function;
        begin const P: Percent := Add(Percent(1), Percent(2)); end."
    ));
    assert!(
        errors.iter().any(|error| error.message.contains("Numeric")),
        "{errors:?}"
    );
}

#[test]
fn subrange_cases_keep_integer_labels_and_use_explicit_finite_coverage() {
    check_ok(&format!("{TYPES} begin const P := Percent(5); case P of when 0..100: null; end case;
        const N: integer := case P of when Percent(5): 1; when 0..4, 6..100: 2; end case;
        const Box := Some(P); case Box of when Some(5): null; when Some(_): null; when None: null; end case; end."));
    let errors = check_errors(&format!(
        "{TYPES} begin const P := Percent(5); const N: integer := case P of when 0..99: 1; end case; end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE),
        "{errors:?}"
    );
}
