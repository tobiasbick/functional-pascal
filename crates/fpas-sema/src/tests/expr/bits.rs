use super::super::{check_errors, check_ok};

#[test]
fn bit_calls_require_explicit_alias_and_integer_signatures() {
    for (call, result) in [
        ("BitAnd(1, 2)", "integer"),
        ("BitOr(-1, 2)", "integer"),
        ("BitXor(1, 2)", "integer"),
        ("BitNot(1)", "integer"),
        ("ShiftLeft(1, 63)", "integer"),
        ("ShiftRight(-1, 63)", "integer"),
    ] {
        check_ok(&format!(
            "program T; uses Std.Bits as Flags; var X: {result} := Flags.{call}; begin null; end program;"
        ));
    }
    for call in [
        "Bits.BitAnd(1)",
        "Bits.BitNot(1, 2)",
        "Bits.ShiftLeft(1.0, 2)",
        "Bits.ShiftRight(1, false)",
        "BitAnd(1, 2)",
        "Std.Bits.BitNot(1)",
    ] {
        check_errors(&format!(
            "program T; uses Std.Bits as Bits; var X: integer := {call}; begin null; end program;"
        ));
    }
    check_errors("program T; var X: integer := Bits.BitNot(1); begin null; end program;");
    check_errors(
        "program T; uses Std.Bits as Bits; const X: integer := Bits.BitNot(1); begin null; end program;",
    );
}

#[test]
fn logical_operators_reject_integers_even_in_unreachable_operands() {
    for expr in [
        "1 and 2",
        "1 or 2",
        "1 xor 2",
        "not 1",
        "false and 1",
        "true or 1",
    ] {
        check_errors(&format!(
            "program T; var X: boolean := {expr}; begin null; end program;"
        ));
    }
    check_ok("program T; var X: boolean := not 3 > 2 and 1 < 2; begin null; end program;");
}
