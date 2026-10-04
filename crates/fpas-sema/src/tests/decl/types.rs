use super::{check_errors, check_ok};

#[test]
fn record_type_valid() {
    check_ok(
        r#"program T;  type Point = record X: real; Y: real; end record; begin null; end program;"#,
    );
}

#[test]
fn enum_type_valid() {
    check_ok(
        r#"program T;  type Color = enum Red; Green; Blue; end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_allows_explicit_i64_max_as_the_last_backing_value() {
    check_ok(
        r#"program T;  type Limit = enum Last = 9223372036854775807; end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_rejects_implicit_backing_value_after_i64_max() {
    let errors = check_errors(
        r#"program T;  type Limit = enum Last = 9223372036854775807; Overflow; end enum; begin null; end program;"#,
    );

    assert!(errors.iter().any(|error| {
        error.code == fpas_diagnostics::codes::SEMA_ENUM_BACKING_VALUE_EXHAUSTED
            && error.message.contains("Limit.Overflow")
    }));
}

#[test]
fn enum_explicit_value_restarts_sequence_after_i64_max() {
    check_ok(
        r#"program T;  type Limit = enum Last = 9223372036854775807; Restart = 0; Next; end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_duplicate_member_rejected() {
    let errors = check_errors(
        r#"program T;  type Color = enum Red; red; end enum; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate enum member error, got: {errors:#?}"
    );
}

#[test]
fn enum_qualified_members_in_scope() {
    check_ok(
        r#"program T;

type Color = enum
  Red;
  Green;
  Blue;
end enum;

const C: Color := Color.Red;

begin
  null;
end program;
"#,
    );
}

#[test]
fn enum_short_variant_names_are_unknown_even_with_an_expected_type() {
    let errors = check_errors(
        r#"program T;  type Color = enum Red; Green; end enum;  type Status = enum Red; Ready; end enum; begin const C: Color := Red; end program;"#,
    );
    assert!(
        errors.iter().any(|error| {
            error.code == fpas_diagnostics::codes::SEMA_UNKNOWN_NAME
                && error.message.contains("Undefined identifier `Red`")
        }),
        "expected rejection of unqualified variant, got: {errors:#?}"
    );
}

#[test]
fn enum_shared_variant_name_does_not_error_when_qualified() {
    check_ok(
        r#"program T;  type Color = enum Red; Green; end enum;  type Status = enum Red; Ready; end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_qualified_variant_names_remain_unambiguous() {
    check_ok(
        r#"program T;  type Color = enum Red; Green; end enum;  type Status = enum Red; Ready; end enum; begin const C: Color := Color.Red; const S: Status := Status.Red; end program;"#,
    );
}

#[test]
fn enum_data_type_valid() {
    check_ok(
        r#"program T;  type Shape = enum Circle(Radius: real); Rectangle(W: real; H: real); end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_data_duplicate_field_rejected() {
    let errors = check_errors(
        r#"program T;  type Shape = enum Circle(Radius: real; radius: integer); end enum; begin null; end program;"#,
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
        r#"program T;  type Token = enum Eof; Number(Value: integer); Word(Text: string); end enum; begin null; end program;"#,
    );
}

#[test]
fn enum_data_construct_valid() {
    check_ok(
        r#"program T;  type Shape = enum Circle(Radius: real); end enum;  const S: Shape := Shape.Circle(5.0); begin null; end program;"#,
    );
}

#[test]
fn enum_data_fieldless_construct_valid() {
    check_ok(
        r#"program T;  type Token = enum Eof; Number(Value: integer); end enum;  const T: Token := Token.Eof; begin null; end program;"#,
    );
}

#[test]
fn enum_data_case_destructure_valid() {
    check_ok(
        r#"program T;

uses Std.Console as Console;

type Shape = enum
  Circle(Radius: real);
  Dot;
end enum;

begin
  const S: Shape := Shape.Circle(1.0);
  case S of
    when Shape.Circle(const R):
      Console.WriteLn(R);
    when Shape.Dot:
      Console.WriteLn('dot');
  end case;
end program;
"#,
    );
}

#[test]
fn enum_data_wrong_arg_count() {
    check_errors(
        r#"program T;  type Shape = enum Circle(Radius: real); end enum;  const S: Shape := Shape.Circle(1.0, 2.0); begin null; end program;"#,
    );
}

#[test]
fn enum_data_wrong_arg_type() {
    check_errors(
        r#"program T;  type Shape = enum Circle(Radius: real); end enum;  const S: Shape := Shape.Circle('text'); begin null; end program;"#,
    );
}

#[test]
fn unknown_type() {
    check_errors(r#"program T;  const X: Foo := 42; begin null; end program;"#);
}

#[test]
fn type_alias_scalar_valid() {
    check_ok(
        r#"program T;  type UserId = integer;  const Id: UserId := 42; begin null; end program;"#,
    );
}

#[test]
fn type_alias_names_are_case_insensitive() {
    check_ok(
        r#"program T;  type UserId = integer;  const Id: userid := 42; begin null; end program;"#,
    );
}

#[test]
fn enum_variant_is_available_through_type_alias() {
    check_ok(
        r#"program T;  type Color = enum Red; Green; end enum;  type PaletteColor = Color;  const C: PaletteColor := PaletteColor.Green; begin null; end program;"#,
    );
}

#[test]
fn type_alias_to_unknown_type() {
    let errors = check_errors(r#"program T;  type Foo = Nonexistent; begin null; end program;"#);
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
        r#"program T;  const Alias: integer := 1;  const X: Alias := 2; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.code == fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE),
        "expected SEMA_UNKNOWN_TYPE, got: {errors:#?}"
    );
}

#[test]
fn record_construction_field_names_are_case_insensitive() {
    check_ok(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

const P: Point := Point(x := 1, y := 2);

begin
  null;
end program;
"#,
    );
}

fn duplicate_record_field_errors(source: &str) -> Vec<crate::SemaError> {
    check_errors(source)
        .into_iter()
        .filter(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION)
        .collect()
}

#[test]
fn record_construction_rejects_exact_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T; type Box = record Value: integer; end record; const N: Box := Box(Value := 1, Value := 2); begin null; end program;"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn record_construction_rejects_case_only_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T; type Box = record Value: integer; end record; const N: Box := Box(Value := 1, value := 2); begin null; end program;"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn typed_record_construction_rejects_exact_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

const P: Point := Point(X := 1, X := 2, Y := 3);

begin
  null;
end program;
"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn typed_record_construction_rejects_case_only_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

const P: Point := Point(X := 1, x := 2, Y := 3);

begin
  null;
end program;
"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn record_update_rejects_exact_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

const P: Point := Point(X := 1, Y := 2);
const Q: Point := P with X := 3; X := 4; end with;

begin
  null;
end program;
"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn record_update_rejects_case_only_duplicate_fields() {
    let errors = duplicate_record_field_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

const P: Point := Point(X := 1, Y := 2);
const Q: Point := P with X := 3; x := 4; end with;

begin
  null;
end program;
"#,
    );
    assert_eq!(errors.len(), 1, "unexpected diagnostics: {errors:#?}");
}

#[test]
fn record_duplicate_field_rejected() {
    let errors = check_errors(
        r#"program T;  type Point = record X: integer; x: integer; end record; begin null; end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
        "expected duplicate record field error, got: {errors:#?}"
    );
}
