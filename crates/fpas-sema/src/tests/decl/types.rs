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
fn enum_shared_variant_name_becomes_ambiguous_at_use_site() {
    let errors = check_errors(
        "program T; \
         type Color = enum Red; Green; end; \
         type Status = enum Red; Ready; end; \
         begin \
           var C: Color := Red \
         end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME
                && error.message.contains("Ambiguous name `Red`")
        }),
        "expected ambiguous short enum variant at use site, got: {errors:#?}"
    );
}

#[test]
fn enum_shared_variant_name_does_not_error_when_qualified() {
    check_ok(
        "program T; \
         type Color = enum Red; Green; end; \
         type Status = enum Red; Ready; end; \
         begin end.",
    );
}

#[test]
fn enum_qualified_variant_names_remain_unambiguous() {
    check_ok(
        "program T; \
         type Color = enum Red; Green; end; \
         type Status = enum Red; Ready; end; \
         begin \
           var C: Color := Color.Red; \
           var S: Status := Status.Red \
         end.",
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
fn enum_variant_is_available_through_type_alias() {
    check_ok(
        "program T; \
         type Color = enum Red; Green; end; \
         type PaletteColor = Color; \
         var C: PaletteColor := PaletteColor.Green; \
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

#[test]
fn static_record_function_valid() {
    check_ok(
        "program T; uses Std.Console; \
         type Point = record \
           X: integer; Y: integer; \
           static function Create(X: integer; Y: integer): Point; \
           begin return record X := X; Y := Y; end end; \
         end; \
         begin \
           var P: Point := Point.Create(3, 4); \
           WriteLn(P.X) \
         end.",
    );
}

#[test]
fn static_record_function_case_insensitive() {
    check_ok(
        "program T; uses Std.Console; \
         type Point = record \
           X: integer; Y: integer; \
           static function Create(X: integer; Y: integer): Point; \
           begin return record X := X; Y := Y; end end; \
         end; \
         begin \
           var P: Point := point.create(1, 2); \
           WriteLn(P.X) \
         end.",
    );
}

#[test]
fn static_record_function_via_alias() {
    check_ok(
        "program T; uses Std.Console; \
         type Point = record \
           X: integer; Y: integer; \
           static function Create(X: integer; Y: integer): Point; \
           begin return record X := X; Y := Y; end end; \
         end; \
         type Alias = Point; \
         begin \
           var P: Alias := Alias.Create(5, 6); \
           WriteLn(P.X) \
         end.",
    );
}

#[test]
fn static_function_with_self_param_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; \
           static function Create(Self: Point; X: integer): Point; \
           begin return Self end; \
         end; \
         begin end.",
    );
    assert!(
        errors.iter().any(
            |error| error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error
                    .message
                    .contains("must not declare a `Self` parameter")
        ),
        "expected Self rejection, got: {errors:#?}"
    );
}

#[test]
fn static_call_through_value_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; Y: integer; \
           static function Create(X: integer; Y: integer): Point; \
           begin return record X := X; Y := Y; end end; \
         end; \
         begin \
           var P: Point := record X := 0; Y := 0; end; \
           var Q: Point := P.Create(1, 2) \
         end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("static function")
        }),
        "expected static-through-value error, got: {errors:#?}"
    );
}

#[test]
fn instance_call_through_type_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; Y: integer; \
           function Sum(Self: Point): integer; \
           begin return Self.X + Self.Y end; \
         end; \
         begin \
           var P: Point := record X := 1; Y := 2; end; \
           var N: integer := Point.Sum(P) \
         end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("instance method")
        }),
        "expected instance-through-type error, got: {errors:#?}"
    );
}

#[test]
fn static_and_instance_duplicate_name_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; \
           static function Sum(X: integer): integer; \
           begin return X end; \
           function Sum(Self: Point): integer; \
           begin return Self.X end; \
         end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate name error, got: {errors:#?}"
    );
}

#[test]
fn static_overload_attempt_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record \
           X: integer; \
           static function Create(X: integer): Point; \
           begin return record X := X; end end; \
           static function Create(X: integer; Y: integer): Point; \
           begin return record X := X; end end; \
         end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected overload rejection, got: {errors:#?}"
    );
}

#[test]
fn static_generic_function_valid() {
    check_ok(
        "program T; uses Std.Console; \
         type Box = record \
           Value: integer; \
           static function Wrap<T>(V: T): T; \
           begin return V end; \
         end; \
         begin \
           WriteLn(Box.Wrap(42)) \
         end.",
    );
}

#[test]
fn static_record_procedure_valid() {
    check_ok(
        "program T; uses Std.Console; \
         type Counter = record \
           static procedure Print(Value: integer); \
           begin WriteLn(Value) end; \
         end; \
         begin \
           Counter.Print(4) \
         end.",
    );
}

#[test]
fn static_record_procedure_via_alias() {
    check_ok(
        "program T; \
         type Counter = record \
           static procedure Reset(Value: integer); \
           begin end; \
         end; \
         type Alias = Counter; \
         begin \
           Alias.Reset(4) \
         end.",
    );
}

#[test]
fn static_procedure_with_self_param_rejected() {
    let errors = check_errors(
        "program T; \
         type Counter = record \
           static procedure Reset(Self: Counter); begin end; \
         end; \
         begin end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error
                    .message
                    .contains("must not declare a `Self` parameter")
        }),
        "expected Self rejection, got: {errors:#?}"
    );
}

#[test]
fn static_procedure_call_through_value_rejected() {
    let errors = check_errors(
        "program T; \
         type Counter = record \
           Value: integer; \
           static procedure Reset(); begin end; \
         end; \
         begin \
           var Value: Counter := record Value := 1; end; \
           Value.Reset() \
         end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("static procedure")
        }),
        "expected static-through-value error, got: {errors:#?}"
    );
}

#[test]
fn static_procedure_cannot_be_used_as_expression() {
    let errors = check_errors(
        "program T; \
         type Counter = record \
           static procedure Reset(); begin end; \
         end; \
         begin var Value: integer := Counter.Reset() end.",
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
                && error.message.contains("does not return a value")
        }),
        "expected procedure-value error, got: {errors:#?}"
    );
}
