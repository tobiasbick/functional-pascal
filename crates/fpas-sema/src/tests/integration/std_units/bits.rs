//! Imports and fixed integer signature checking for `Std.Bits`.

use super::{check_errors, check_ok};

#[test]
fn bits_supports_imported_short_and_case_insensitive_qualified_names() {
    check_ok(
        "program Bits; uses sTd.bItS; begin const A: integer := BitAnd(12, 10); const B: integer := std.bits.shiftleft(A, 1); const C: integer := BitNot(BitOr(B, BitXor(1, 2))); end.",
    );
}

#[test]
fn bits_requires_an_explicit_import() {
    for name in ["BitAnd", "Std.Bits.BitAnd"] {
        let errors = check_errors(&format!(
            "program Missing; begin const A: integer := {name}(1, 2); end."
        ));
        assert!(!errors.is_empty(), "missing import accepted for {name}");
    }
}

#[test]
fn bits_rejects_wrong_arity_and_types_for_every_function() {
    for name in [
        "BitAnd",
        "BitOr",
        "BitXor",
        "BitNot",
        "ShiftLeft",
        "ShiftRight",
    ] {
        let cases = if name == "BitNot" {
            vec!["", "1, 2", "true", "1.0", "'1'"]
        } else {
            vec![
                "", "1", "1, 2, 3", "true, 1", "1, true", "1.0, 1", "1, 1.0", "'1', 1", "1, '1'",
            ]
        };
        for arguments in cases {
            let errors = check_errors(&format!(
                "program Invalid; uses Std.Bits; begin const A: integer := {name}({arguments}); end."
            ));
            assert!(!errors.is_empty(), "{name}({arguments}) was accepted");
        }
    }
}
