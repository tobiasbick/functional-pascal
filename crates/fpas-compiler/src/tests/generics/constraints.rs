use super::*;

// All constraint tests involving user-defined generic types now produce parse errors.
// Constraints on generic functions remain valid — see functions/multi_param.rs.

#[test]
fn constrained_generic_record_produces_parse_error() {
    parse_fails(
        "\
program T;
type Ordered<T: Comparable> = record Value: T; end;
begin end.",
    );
}

#[test]
fn constrained_generic_function_comparable_is_valid() {
    compile_ok(
        "\
program T;
uses Std.Console;
function Max<T: Comparable>(A: T; B: T): T;
begin
  if A > B then return A else return B
end;
begin
  WriteLn(Max(3, 7))
end.",
    );
}
