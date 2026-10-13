//! User diagnostics for reachable function exits without a value.

use super::super::{assert_succeeds, parse_ok};

#[test]
fn reachable_missing_returns_report_a_user_diagnostic() {
    for body in [
        "null;",
        "if Value > 0 then return Value; end if;",
        "case Value of when 0: return 0; when const N if N > 0: return N; when const N if N < 0: return N; else null; end case;",
    ] {
        for contract in ["", "ensures (Returned): Returned > 0;"] {
            let program = parse_ok(&format!(
                "program T; function Checked(Value: integer): integer;
                 {contract} begin {body} end function;
                 begin const ResultValue: integer := Checked(0); end."
            ));
            let errors = crate::compile(&program).expect_err("reachable missing return");
            assert!(
                errors.iter().any(|error| {
                    error.code == fpas_diagnostics::codes::COMPILE_FUNCTION_MISSING_RETURN
                        && error.help.is_some()
                }),
                "{contract}: {body}: {errors:?}"
            );
        }
    }
}

#[test]
fn complete_branches_do_not_require_an_unreachable_return() {
    assert_succeeds(
        "program T; uses Std.Test;
         function Checked(Value: integer): integer;
         begin
           case Value of
             when 0: return 10;
             when const N if N > 0: return 20;
             else return 30;
           end case;
         end function;
         begin
           AssertEquals(10, Checked(0));
           AssertEquals(20, Checked(1));
           AssertEquals(30, Checked(-1));
         end.",
    );
}
