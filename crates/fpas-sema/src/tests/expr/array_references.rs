//! Native array APIs require the same explicit storage arguments as source calls.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH};

#[test]
fn array_mutations_check_writable_selected_storage_and_forwarded_modes() {
    check_ok(
        r#"program T; uses Std.Arrays as Arrays;
type Box = record Items: array of (integer); end record;
procedure Append(var Items: array of (integer)); begin Arrays.Push(var Items, 2); end procedure;
begin  var Data: Box := Box(Items := [1]); Append(var Data.Items);
   var Mapping: dict of (string, array of (integer)) := ['key': [1]];
  discard Arrays.Pop(var Mapping['key']); end program;"#,
    );
}

#[test]
fn array_mutations_reject_missing_markers_readonly_roots_and_wrong_types() {
    for (source, code) in [
        (
            r#"begin  var A: array of (integer) := [1]; Arrays.Push(A, 2); end program;"#,
            SEMA_TYPE_MISMATCH,
        ),
        (
            r#"begin  var A: array of (integer) := [1]; discard Arrays.Pop(A); end program;"#,
            SEMA_TYPE_MISMATCH,
        ),
        (
            "const A: array of (integer) := [1]; begin Arrays.Push(var A, 2); end program;",
            SEMA_IMMUTABLE_ASSIGNMENT,
        ),
        (
            "procedure P(A: array of (integer)); begin discard Arrays.Pop(var A); end procedure; begin null; end program;",
            SEMA_IMMUTABLE_ASSIGNMENT,
        ),
        (
            r#"begin  var A: integer := 1; Arrays.Push(var A, 2); end program;"#,
            SEMA_TYPE_MISMATCH,
        ),
        (
            r#"begin  var A: array of (integer) := [1]; Arrays.Push(var A, 'x'); end program;"#,
            SEMA_TYPE_MISMATCH,
        ),
        (
            r#"begin  var A: array of (integer) := [1]; const T: task := go Arrays.Pop(var A); end program;"#,
            SEMA_TYPE_MISMATCH,
        ),
    ] {
        let errors = check_errors(&format!("program T; uses Std.Arrays as Arrays; {source}"));
        assert!(
            errors.iter().any(|error| error.code == code),
            "{source}: {errors:#?}"
        );
    }
}

#[test]
fn var_first_parameters_cannot_receive_an_implicit_value_receiver() {
    let errors = check_errors(
        r#"program T; procedure Increase(var Value: integer); begin Value := Value + 1; end procedure; begin  var A: integer := 1; A.Increase(); end program;"#,
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
            && error
                .message
                .contains("`.Increase` requires a record value")),
        "{errors:#?}"
    );
}
