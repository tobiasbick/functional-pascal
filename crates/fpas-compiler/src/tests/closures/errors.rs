use super::*;
#[test]
fn map_wrong_arity_compile_error() {
    let err = compile_err(
        "\
program MapBadArity;
uses Std.Array;
begin
  var A: array of integer := [1, 2];
  var B: array of integer := Map(A)
end.",
    );
    assert!(
        err.message.contains("expects 2 arguments")
            || err.message.contains("expected 2")
            || err.message.contains("Map"),
        "unexpected error: {err:?}"
    );
}
#[test]
fn reduce_wrong_arity_compile_error() {
    let err = compile_err(
        "\
program ReduceBadArity;
uses Std.Array;
begin
  var A: array of integer := [1, 2];
  var B: integer := Reduce(A, 0)
end.",
    );
    assert!(
        err.message.contains("expects 3 arguments")
            || err.message.contains("expected 3")
            || err.message.contains("Reduce"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn function_type_mismatch_in_assignment() {
    let err = compile_err(
        "\
program FuncTypeMismatch;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var F: function(X: integer): integer := Add
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}
