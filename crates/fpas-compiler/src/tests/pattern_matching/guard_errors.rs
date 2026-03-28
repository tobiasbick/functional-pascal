use super::*;

// ===========================================================================
// Guard clauses — negative tests (sema errors)
// ===========================================================================

#[test]
fn guard_non_boolean_expression_is_error() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if 42:
      Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('ok')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_NON_BOOLEAN_CONDITION
    );
}

#[test]
fn guard_string_expression_is_error() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if 'hello':
      Std.Console.WriteLn('bad')
  else
    Std.Console.WriteLn('ok')
  end
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_NON_BOOLEAN_CONDITION
    );
}
