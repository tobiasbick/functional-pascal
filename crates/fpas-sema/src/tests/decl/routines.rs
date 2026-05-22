use super::{check_errors, check_ok};

#[test]
fn function_valid() {
    check_ok(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin return A + B end; \
         begin end.",
    );
}

#[test]
fn function_return_type_mismatch() {
    check_errors(
        "program T; \
         function GetNum(): integer; \
         begin return true end; \
         begin end.",
    );
}

#[test]
fn function_duplicate_definition_rejected() {
    let errors = check_errors(
        "program T; \
         function F(): integer; \
         begin return 1 end; \
         function F(): integer; \
         begin return 2 end; \
         begin end.",
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
        "program T; \
         function F(X: integer; x: integer): integer; \
         begin return X end; \
         begin end.",
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
        "program T; \
         function F<T, t>(Value: T): T; \
         begin return Value end; \
         begin end.",
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
        "program T; \
         procedure DoStuff(X: integer); \
         begin return end; \
         begin end.",
    );
}

#[test]
fn procedure_return_value_error() {
    check_errors(
        "program T; \
         procedure DoStuff(); \
         begin return 42 end; \
         begin end.",
    );
}

#[test]
fn function_missing_return_value() {
    check_errors(
        "program T; \
         function GetNum(): integer; \
         begin return end; \
         begin end.",
    );
}

#[test]
fn nested_function_scope() {
    check_ok(
        "program T; \
         function Outer(): integer; \
           function Inner(): integer; \
           begin return 1 end; \
         begin return Inner() end; \
         begin end.",
    );
}

#[test]
fn mutable_param() {
    check_ok(
        "program T; \
         procedure Inc(mutable X: integer); \
         begin X := X + 1 end; \
         begin end.",
    );
}

#[test]
fn generic_function_valid() {
    check_ok(
        "program T; \
         function Identity<T>(Value: T): T; \
         begin return Value end; \
         var X: integer := Identity(42); \
         begin end.",
    );
}

#[test]
fn generic_procedure_valid() {
    check_ok(
        "program T; uses Std.Console; \
         procedure Print<T>(Value: T); \
         begin WriteLn(Value) end; \
         begin Print(42) end.",
    );
}

#[test]
fn generic_function_reused_type_param_requires_same_concrete_type() {
    check_errors(
        "program T; \
         function PickFirst<T>(A: T; B: T): T; \
         begin return A end; \
         begin \
           var X: integer := PickFirst(1, true) \
         end.",
    );
}

#[test]
fn generic_function_numeric_constraint_allows_arithmetic() {
    check_ok(
        "program T; \
         function Add<T: Numeric>(A: T; B: T): T; \
         begin return A + B end; \
         begin Add(1, 2) end.",
    );
}

#[test]
fn generic_function_numeric_constraint_allows_negate() {
    check_ok(
        "program T; \
         function Neg<T: Numeric>(X: T): T; \
         begin return -X end; \
         begin Neg(5) end.",
    );
}

#[test]
fn generic_function_comparable_constraint_allows_lt() {
    check_ok(
        "program T; \
         function IsLess<T: Comparable>(A: T; B: T): boolean; \
         begin return A < B end; \
         begin IsLess(1, 2) end.",
    );
}

#[test]
fn generic_function_unconstrained_rejects_arithmetic() {
    let errors = check_errors(
        "program T; \
         function Add<T>(A: T; B: T): T; \
         begin return A + B end; \
         begin Add(1, 2) end.",
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
        "program T; \
         function Compare<T: Comparable>(A: T; B: T): boolean; \
         begin return A = B end; \
         begin Compare([1], [2]) end.",
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
        "program T; \
         function Add<T: Numeric>(A: T; B: T): T; \
         begin return A + B end; \
         begin Add('a', 'b') end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_CONSTRAINT_VIOLATION),
        "expected SEMA_CONSTRAINT_VIOLATION at call site, got: {errors:#?}"
    );
}
