use super::super::{check_errors, check_ok};

#[test]
fn for_valid() {
    check_ok(r#"program T; begin for I: integer := 0 to 9 do return; end for; end program;"#);
}

#[test]
fn for_var_is_immutable() {
    check_errors(r#"program T; begin for I: integer := 0 to 9 do I := 5; end for; end program;"#);
}

#[test]
fn for_non_ordinal_error() {
    check_errors(r#"program T; begin for X: real := 0.0 to 1.0 do return; end for; end program;"#);
}

#[test]
fn for_start_type_mismatch() {
    check_errors(
        r#"program T; begin for I: integer := 'hello' to 10 do return; end for; end program;"#,
    );
}

#[test]
fn for_end_type_mismatch() {
    check_errors(
        r#"program T; begin for I: integer := 1 to 'world' do return; end for; end program;"#,
    );
}

#[test]
fn for_var_not_accessible_after_loop() {
    check_errors(
        r#"program T; begin for I: integer := 1 to 5 do Std.Console.WriteLn(I); end for; Std.Console.WriteLn(I); end program;"#,
    );
}

#[test]
fn for_in_valid() {
    check_ok(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: integer in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_non_array_error() {
    check_errors(
        r#"program T; begin const N: integer := 0; for X: integer in N do return; end for; end program;"#,
    );
}

#[test]
fn for_in_element_type_mismatch() {
    check_errors(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: string in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_var_is_immutable() {
    check_errors(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: integer in Arr do X := 5; end for; end program;"#,
    );
}

#[test]
fn for_in_var_not_accessible_after_loop() {
    check_errors(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: integer in Arr do Std.Console.WriteLn(X); end for; Std.Console.WriteLn(X); end program;"#,
    );
}

#[test]
fn for_in_on_string_error() {
    check_errors(
        r#"program T; begin const S: string := 'hello'; for C: string in S do return; end for; end program;"#,
    );
}

#[test]
fn for_in_on_boolean_error() {
    check_errors(
        r#"program T; begin const B: boolean := true; for X: boolean in B do return; end for; end program;"#,
    );
}

#[test]
fn for_in_on_real_error() {
    check_errors(
        r#"program T; begin const R: real := 3.14; for X: real in R do return; end for; end program;"#,
    );
}

#[test]
fn for_in_element_type_mismatch_string_for_integer_array() {
    check_errors(
        r#"program T; begin const Arr: array of (string) := ['a']; for X: integer in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_element_type_mismatch_real_for_integer_array() {
    check_errors(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: real in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_element_type_mismatch_boolean_for_string_array() {
    check_errors(
        r#"program T; begin const Arr: array of (string) := ['x']; for X: boolean in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_valid_boolean_array() {
    check_ok(
        r#"program T; begin const Arr: array of (boolean) := [true, false]; for X: boolean in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_valid_real_array() {
    check_ok(
        r#"program T; begin const Arr: array of (real) := [1.0]; for X: real in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_valid_string_array() {
    check_ok(
        r#"program T; begin const Arr: array of (string) := ['a']; for S: string in Arr do return; end for; end program;"#,
    );
}

#[test]
fn for_in_assign_to_loop_var_in_nested_block() {
    check_errors(
        r#"program T; begin const Arr: array of (integer) := [1]; for X: integer in Arr do begin X := 99; end; end for; end program;"#,
    );
}

// ---------------------------------------------------------------------------
// for-in over dict — yields keys in insertion order
// ---------------------------------------------------------------------------

#[test]
fn for_in_dict_valid_string_key() {
    check_ok(
        r#"program T;
         begin const D: dict of (string, integer) := ['a': 1]; for K: string in D do return; end for; end program;"#,
    );
}

#[test]
fn for_in_dict_valid_integer_key() {
    check_ok(
        r#"program T; begin const D: dict of (integer, boolean) := [1: true]; for K: integer in D do return; end for; end program;"#,
    );
}

#[test]
fn for_in_dict_key_type_mismatch() {
    check_errors(
        r#"program T; begin const D: dict of (string, integer) := ['a': 1]; for K: integer in D do return; end for; end program;"#,
    );
}

#[test]
fn for_in_dict_var_is_immutable() {
    check_errors(
        r#"program T; begin const D: dict of (string, integer) := ['a': 1]; for K: string in D do K := 'x'; end for; end program;"#,
    );
}

#[test]
fn for_in_dict_var_not_accessible_after_loop() {
    check_errors(
        r#"program T; begin const D: dict of (string, integer) := ['a': 1]; for K: string in D do return; end for; Std.Console.WriteLn(K); end program;"#,
    );
}
