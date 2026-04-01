use super::*;

// Tests for negative (error) cases in generic functions.

#[test]
fn generic_function_type_param_not_in_scope_outside() {
    // T should not leak outside the generic function.
    let err = compile_err(
        "\
program T;
function Id<T>(V: T): T;
begin
  return V
end;
begin
  var X: T := 42
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

// Generic type definitions (records, enums) now produce parse errors.
#[test]
fn generic_record_definition_produces_parse_error() {
    parse_fails(
        "\
program T;
type Box<T> = record Value: T; end;
begin end.",
    );
}
