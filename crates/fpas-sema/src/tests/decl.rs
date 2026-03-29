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
fn function_forward_valid() {
    check_ok(
        "program T; \
         function F(): integer; forward; \
         function F(): integer; \
         begin return 1 end; \
         begin end.",
    );
}

#[test]
fn function_forward_requires_implementation() {
    let errors = check_errors(
        "program T; \
         function F(): integer; forward; \
         begin end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME
                && error.message.contains("forward")
        }),
        "expected missing forward implementation error, got: {errors:#?}"
    );
}

#[test]
fn function_forward_signature_must_match() {
    let errors = check_errors(
        "program T; \
         function F(X: integer): integer; forward; \
         function F(X: integer): string; \
         begin return 'oops' end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH),
        "expected forward signature mismatch, got: {errors:#?}"
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
