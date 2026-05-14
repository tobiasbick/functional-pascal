use super::{check_errors, check_ok};

#[test]
fn record_type_valid() {
    check_ok("program T; type Point = record X: real; Y: real; end; begin end.");
}

#[test]
fn enum_type_valid() {
    check_ok("program T; type Color = enum Red; Green; Blue; end; begin end.");
}

#[test]
fn enum_duplicate_member_rejected() {
    let errors = check_errors("program T; type Color = enum Red; red; end; begin end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate enum member error, got: {errors:#?}"
    );
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

#[test]
fn enum_data_type_valid() {
    check_ok(
        "program T; \
         type Shape = enum Circle(Radius: real); Rectangle(W: real; H: real); end; \
         begin end.",
    );
}

#[test]
fn enum_data_duplicate_field_rejected() {
    let errors = check_errors(
        "program T; \
         type Shape = enum Circle(Radius: real; radius: integer); end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate enum field error, got: {errors:#?}"
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
fn record_literal_field_names_are_case_insensitive() {
    check_ok(
        "program T; \
         type Point = record X: integer; Y: integer; end; \
         var P: Point := record x := 1; y := 2; end; \
         begin end.",
    );
}

#[test]
fn record_duplicate_field_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; x: integer; end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate record field error, got: {errors:#?}"
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

#[test]
fn record_duplicate_method_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; \
           function Sum(Self: Point): integer; \
           begin return Self.X end; \
           function sum(Self: Point): integer; \
           begin return Self.X end; \
         end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate record method error, got: {errors:#?}"
    );
}