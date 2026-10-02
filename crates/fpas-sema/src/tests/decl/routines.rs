use super::{check_errors, check_ok};

#[test]
fn function_valid() {
    check_ok(
        r#"program T; function Add(A: integer; B: integer): integer; begin return A + B; end function; begin null; end program;"#,
    );
}

#[test]
fn function_return_type_mismatch() {
    check_errors(
        r#"program T; function GetNum(): integer; begin return true; end function; begin null; end program;"#,
    );
}

#[test]
fn function_duplicate_definition_rejected() {
    let errors = check_errors(
        r#"program T; function F(): integer; begin return 1; end function; function F(): integer; begin return 2; end function; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate routine error, got: {errors:#?}"
    );
}

#[test]
fn function_duplicate_parameter_rejected() {
    let errors = check_errors(
        r#"program T; function F(X: integer; x: integer): integer; begin return X; end function; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate parameter error, got: {errors:#?}"
    );
}

#[test]
fn function_duplicate_type_parameter_rejected() {
    let errors = check_errors(
        r#"program T; function F<T, t>(Value: T): T; begin return Value; end function; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate type parameter error, got: {errors:#?}"
    );
}

#[test]
fn procedure_valid() {
    check_ok(
        r#"program T; procedure DoStuff(X: integer); begin return; end procedure; begin null; end program;"#,
    );
}

#[test]
fn procedure_return_value_error() {
    check_errors(
        r#"program T; procedure DoStuff(); begin return 42; end procedure; begin null; end program;"#,
    );
}

#[test]
fn function_missing_return_value() {
    check_errors(
        r#"program T; function GetNum(): integer; begin return; end function; begin null; end program;"#,
    );
}

#[test]
fn nested_function_scope() {
    check_ok(
        r#"program T; function Outer(): integer; function Inner(): integer; begin return 1; end function; begin return Inner(); end function; begin null; end program;"#,
    );
}

#[test]
fn nested_function_captures_enclosing_body_local() {
    check_ok(
        r#"program T; function Make(): function(Value: integer): integer; function Add(Value: integer): integer; begin return Value + Offset; end function; begin var Offset: integer := 7; return Add; end function; begin null; end program;"#,
    );
}

#[test]
fn nested_function_does_not_see_inner_block_locals() {
    let errors = check_errors(
        r#"program T; function Outer(): integer; function Inner(): integer; begin return Hidden; end function; begin begin var Hidden: integer := 1; return Inner(); end; end function; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Undefined identifier `Hidden`")),
        "expected inner-block local to stay hidden, got {errors:#?}"
    );
}

#[test]
fn mutable_param() {
    check_ok(
        r#"program T; procedure Inc(mutable X: integer); begin X := X + 1; end procedure; begin null; end program;"#,
    );
}

#[test]
fn generic_function_valid() {
    check_ok(
        r#"program T; function Identity<T>(Value: T): T; begin return Value; end function;  var X: integer := Identity(42); begin null; end program;"#,
    );
}

#[test]
fn generic_callback_returning_recursive_record_is_valid() {
    check_ok(
        r#"program T;  type Element = record Text: string; Children: array of Element; end record; type Model = record Count: integer; end record; function View(State: Model): Element; begin return record Text := 'root'; Children := []; end record; end function; function Render<TModel>(State: TModel; ViewFn: function(State: TModel): Element): Element; begin return ViewFn(State); end function; begin var Root: Element := Render(record Count := 1; end record, View); end program;"#,
    );
}

#[test]
fn generic_procedure_valid() {
    check_ok(
        r#"program T;  uses Std.Console as Console; procedure Print<T>(Value: T); begin Console.WriteLn(Value); end procedure; begin Print(42); end program;"#,
    );
}

#[test]
fn generic_function_reused_type_param_requires_same_concrete_type() {
    check_errors(
        r#"program T; function PickFirst<T>(A: T; B: T): T; begin return A; end function; begin var X: integer := PickFirst(1, true); end program;"#,
    );
}

#[test]
fn generic_function_numeric_constraint_allows_arithmetic() {
    check_ok(
        r#"program T; function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function; begin Add(1, 2); end program;"#,
    );
}

#[test]
fn generic_function_numeric_constraint_allows_negate() {
    check_ok(
        r#"program T; function Neg<T: Numeric>(X: T): T; begin return -X; end function; begin Neg(5); end program;"#,
    );
}

#[test]
fn generic_function_comparable_constraint_allows_lt() {
    check_ok(
        r#"program T; function IsLess<T: Comparable>(A: T; B: T): boolean; begin return A < B; end function; begin IsLess(1, 2); end program;"#,
    );
}

#[test]
fn generic_function_unconstrained_rejects_arithmetic() {
    let errors = check_errors(
        r#"program T; function Add<T>(A: T; B: T): T; begin return A + B; end function; begin Add(1, 2); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected SEMA_TYPE_MISMATCH for arithmetic on unconstrained T, got: {errors:#?}"
    );
}

#[test]
fn generic_function_constraint_violation_at_call_site() {
    let errors = check_errors(
        r#"program T; function Compare<T: Comparable>(A: T; B: T): boolean; begin return A = B; end function; begin Compare([1], [2]); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION),
        "expected SEMA_CONSTRAINT_VIOLATION at call site, got: {errors:#?}"
    );
}

#[test]
fn generic_function_numeric_violation_at_call_site() {
    let errors = check_errors(
        r#"program T; function Add<T: Numeric>(A: T; B: T): T; begin return A + B; end function; begin Add('a', 'b'); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION),
        "expected SEMA_CONSTRAINT_VIOLATION at call site, got: {errors:#?}"
    );
}
