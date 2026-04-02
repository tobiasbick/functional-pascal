use super::{check_errors, check_ok};

#[test]
fn const_valid() {
    check_ok("program T; const Pi: real := 3.14; begin end.");
}

#[test]
fn const_can_reference_previous_const() {
    check_ok("program T; const A: integer := 40; B: integer := A + 2; begin end.");
}

#[test]
fn const_type_mismatch() {
    check_errors("program T; const X: integer := 3.14; begin end.");
}

#[test]
fn const_initializer_must_be_compile_time_known() {
    let errors = check_errors(
        "program T; \
         function FortyTwo(): integer; \
         begin return 42 end; \
         const X: integer := FortyTwo(); \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION }),
        "expected non-constant-expression diagnostic, got: {errors:#?}"
    );
}

#[test]
fn const_initializer_cannot_read_variable() {
    let errors = check_errors(
        "program T; \
         var Seed: integer := 1; \
         const X: integer := Seed; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION }),
        "expected non-constant-expression diagnostic, got: {errors:#?}"
    );
}

#[test]
fn var_valid() {
    check_ok("program T; var X: integer := 42; begin end.");
}

#[test]
fn var_type_mismatch() {
    check_errors("program T; var X: integer := true; begin end.");
}

#[test]
fn mutable_var_valid() {
    check_ok("program T; mutable var X: integer := 0; begin end.");
}

#[test]
fn duplicate_variable() {
    check_errors("program T; var X: integer := 1; var X: integer := 2; begin end.");
}

#[test]
fn duplicate_variable_differs_only_by_case_rejected() {
    let errors = check_errors("program T; var X: integer := 1; var x: integer := 2; begin end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate variable error, got: {errors:#?}"
    );
}

#[test]
fn record_type_valid() {
    check_ok("program T; type Point = record X: real; Y: real; end; begin end.");
}

#[test]
fn enum_type_valid() {
    check_ok("program T; type Color = enum Red; Green; Blue; end; begin end.");
}

#[test]
fn enum_members_in_scope() {
    check_ok(
        "program T; \
         type Color = enum Red; Green; Blue; end; \
         var C: Color := Red; \
         begin end.",
    );
}

// ── Enums with Associated Data ──────────────────────────────

#[test]
fn enum_data_type_valid() {
    check_ok(
        "program T; \
         type Shape = enum Circle(Radius: real); Rectangle(W: real; H: real); end; \
         begin end.",
    );
}

#[test]
fn enum_data_mixed_valid() {
    check_ok(
        "program T; \
         type Token = enum Eof; Number(Value: integer); Word(Text: string); end; \
         begin end.",
    );
}

#[test]
fn enum_data_construct_valid() {
    check_ok(
        "program T; \
         type Shape = enum Circle(Radius: real); end; \
         var S: Shape := Shape.Circle(5.0); \
         begin end.",
    );
}

#[test]
fn enum_data_fieldless_construct_valid() {
    check_ok(
        "program T; \
         type Token = enum Eof; Number(Value: integer); end; \
         var T: Token := Token.Eof; \
         begin end.",
    );
}

#[test]
fn enum_data_case_destructure_valid() {
    check_ok(
        "program T; uses Std.Console; \
         type Shape = enum Circle(Radius: real); Dot; end; \
         begin \
           var S: Shape := Shape.Circle(1.0); \
           case S of \
             Shape.Circle(R): WriteLn(R); \
             Shape.Dot: WriteLn('dot') \
           end \
         end.",
    );
}

#[test]
fn enum_data_wrong_arg_count() {
    check_errors(
        "program T; \
         type Shape = enum Circle(Radius: real); end; \
         var S: Shape := Shape.Circle(1.0, 2.0); \
         begin end.",
    );
}

#[test]
fn enum_data_wrong_arg_type() {
    check_errors(
        "program T; \
         type Shape = enum Circle(Radius: real); end; \
         var S: Shape := Shape.Circle('text'); \
         begin end.",
    );
}

#[test]
fn unknown_type() {
    check_errors("program T; var X: Foo := 42; begin end.");
}

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

// ── Generics (sema-level) ──────────────────────────────────

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

// ── Constraints (sema-level) ───────────────────────────────
// Constraints on user-defined generic types are gone (generic type
// definitions were removed). Constraint tests live in the generic
// function tests below.

// ── Type Aliases (sema-level) ──────────────────────────────

#[test]
fn type_alias_scalar_valid() {
    check_ok(
        "program T; \
         type UserId = integer; \
         var Id: UserId := 42; \
         begin end.",
    );
}

#[test]
fn type_alias_names_are_case_insensitive() {
    check_ok(
        "program T; \
         type UserId = integer; \
         var Id: userid := 42; \
         begin end.",
    );
}

#[test]
fn type_alias_to_unknown_type() {
    let errors = check_errors(
        "program T; \
         type Foo = Nonexistent; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE),
        "expected SEMA_UNKNOWN_TYPE, got: {errors:#?}"
    );
}

#[test]
fn value_name_cannot_be_used_as_type() {
    let errors = check_errors(
        "program T; \
         var Alias: integer := 1; \
         var X: Alias := 2; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE),
        "expected SEMA_UNKNOWN_TYPE, got: {errors:#?}"
    );
}

#[test]
fn variable_names_are_case_insensitive() {
    check_ok(
        "program T; \
         var X: integer := 1; \
         var Y: integer := x; \
         begin end.",
    );
}

#[test]
fn record_literal_field_names_are_case_insensitive() {
    check_ok(
        "program T; \
         type Point = record X: integer; Y: integer; end; \
         var P: Point := record x := 1; y := 2; end; \
         begin end.",
    );
}

#[test]
fn record_method_valid() {
    check_ok(
        "program T; uses Std.Console; \
         type Point = record \
           X: integer; Y: integer; \
           function Sum(Self: Point): integer; \
           begin return Self.X + Self.Y end; \
         end; \
         begin \
           var P: Point := record X := 3; Y := 7; end; \
           WriteLn(P.Sum()) \
         end.",
    );
}

#[test]
fn record_method_names_are_case_insensitive() {
    check_ok(
        "program T; uses Std.Console; \
                 type Point = record \
                     X: integer; \
                     function Sum(Self: Point): integer; \
                     begin return Self.X end; \
                 end; \
                 begin \
                     var P: Point := record X := 3; end; \
                     WriteLn(P.sum()) \
                 end.",
    );
}

// ── Constraint-aware operators in generic function bodies ───

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

// ── Constraint validation at function call sites ───────────

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
