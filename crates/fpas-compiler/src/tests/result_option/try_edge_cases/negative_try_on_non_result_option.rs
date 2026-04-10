use super::super::compile_err;
// ── Negative: try on non-Result/Option ──────────────────────────────────

#[test]
fn try_on_integer_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of integer, string;
begin
  var X: integer := try 42;
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_on_string_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of string, string;
begin
  var X: string := try 'hello';
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_on_boolean_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of boolean, string;
begin
  var X: boolean := try true;
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn try_result_in_plain_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): integer;
begin
  var X: integer := try Ok(42);
  return X
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("returns `Integer`") || err.message.contains("returns `integer`"),
        "expected enclosing return type mismatch, got: {err:?}"
    );
}

#[test]
fn try_option_in_plain_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): integer;
begin
  var X: integer := try Some(42);
  return X
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("returns `Integer`") || err.message.contains("returns `integer`"),
        "expected enclosing return type mismatch, got: {err:?}"
    );
}

#[test]
fn try_result_in_option_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Option of integer;
begin
  var X: integer := try Ok(42);
  return Some(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Option") && err.message.contains("Result"),
        "expected Result/Option mismatch, got: {err:?}"
    );
}

#[test]
fn try_option_in_result_function_is_compile_error() {
    let err = compile_err(
        "program T;
function Foo(): Result of integer, string;
begin
  var X: integer := try Some(42);
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Option") && err.message.contains("Result"),
        "expected Option/Result mismatch, got: {err:?}"
    );
}

#[test]
fn try_result_error_type_mismatch_is_compile_error() {
    let err = compile_err(
        "program T;
function Inner(): Result of integer, string;
begin
  return Error('boom')
end;
function Outer(): Result of integer, integer;
begin
  var X: integer := try Inner();
  return Ok(X)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Result") && err.message.contains("Outer"),
        "expected Result error-type mismatch, got: {err:?}"
    );
}

#[test]
fn try_in_procedure_is_compile_error() {
    let err = compile_err(
        "program T;
procedure Foo();
begin
  var X: integer := try Some(42)
end;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
    assert!(
        err.message.contains("Procedure `Foo`"),
        "expected procedure-specific diagnostic, got: {err:?}"
    );
}
