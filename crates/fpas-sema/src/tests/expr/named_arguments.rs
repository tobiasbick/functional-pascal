//! Named call arguments and pattern fields: static labels and complete mappings.
//!
//! Documentation: `docs/pascal/language/functions/parameters.md`

use super::{check_errors, check_ok};
use crate::analyze_with_types;
use fpas_diagnostics::codes::{
    SEMA_INVALID_NAMED_ARGUMENT, SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED, SEMA_TYPE_MISMATCH,
};

const ROUTINES: &str = r#"
type Point = record
  X: integer;
  Y: integer;

  function Moved(Self: Point; Dx: integer; Dy: integer): Point;
  begin
    return Self with X := Self.X + Dx; Y := Self.Y + Dy; end with;
  end function;

  static function Create(X: integer; Y: integer): Point;
  begin
    return Point( X := X, Y := Y );
  end function;
end record;

type Shape = enum
  Circle(Radius: real);
  Rect(Width: real; Height: real);
end enum;

function Sub(Left: integer; Right: integer): integer;
begin
  return Left - Right;
end function;

procedure Show(Text: string; Count: integer);
begin
end procedure;
"#;

fn program(body: &str) -> String {
    format!(
        "program T;\nuses Std.Console as Console, Std.Math as Math;\n{ROUTINES}\nbegin\n{body}\nend."
    )
}

fn single_error_code(body: &str) -> fpas_diagnostics::DiagnosticCode {
    let errors = check_errors(&program(body));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    errors[0].code
}

#[test]
fn named_arguments_map_routines_methods_and_standard_routines() {
    check_ok(&program(
        r#"
  const A: integer := Sub(Right := 1, left := 2);
  Show(Count := 1, Text := 'x');
  const P: Point := Point.Create(Y := 2, X := 1);
  const Q: Point := P.Moved(Dy := 1, Dx := 2);
  const S: string := 'x'.PadLeft(PadChar := '.', Width := 3);
"#,
    ));
}

#[test]
fn named_calls_record_parameter_order_by_first_written_argument() {
    let (parsed, parse_errors) = fpas_parser::parse(&program(
        "  const A: integer := Sub(Right := 1, Left := 2);",
    ));
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let metadata = analyze_with_types(&parsed);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    let orders = metadata
        .named_argument_orders
        .values()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(orders, vec![vec![1, 0]]);
}

#[test]
fn named_argument_values_are_checked_against_their_parameter() {
    let errors = check_errors(&program(
        "  const A: integer := Sub(Right := 'x', Left := 2);",
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("argument `Right`")),
        "{errors:#?}"
    );
}

#[test]
fn invalid_named_mappings_list_the_parameters() {
    for body in [
        "  const A: integer := Sub(Left := 1, Rigth := 2);",
        "  const A: integer := Sub(Left := 1, Left := 2);",
        "  const A: integer := Sub(Left := 1);",
    ] {
        assert_eq!(
            single_error_code(body),
            SEMA_INVALID_NAMED_ARGUMENT,
            "{body}"
        );
    }
    let errors = check_errors(&program(
        "  const A: integer := Sub(Left := 1, Rigth := 2);",
    ));
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("Left, Right")),
        "{errors:#?}"
    );
}

#[test]
fn positional_only_targets_reject_named_arguments() {
    for body in [
        "  Console.WriteLn(Text := 'x');",
        "  const S: string := '%d'.Format(Value := 1);",
    ] {
        assert_eq!(
            single_error_code(body),
            SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED,
            "{body}"
        );
    }
}

#[test]
fn named_arguments_construct_enum_variants_by_field_name() {
    check_ok(&program(
        r#"
  const A: Shape := Shape.Rect(Height := 1.0, width := 2.0);
  const B: Shape := Shape.Circle(Radius := 1.0);
"#,
    ));
}

#[test]
fn invalid_named_variant_fields_use_field_wording() {
    for (body, message) in [
        (
            "  const S: Shape := Shape.Rect(Width := 1.0, Heigth := 2.0);",
            "`Shape.Rect` has no field `Heigth`",
        ),
        (
            "  const S: Shape := Shape.Rect(Width := 1.0, Width := 2.0);",
            "Field `Width` of `Shape.Rect` is named more than once",
        ),
        (
            "  const S: Shape := Shape.Rect(Width := 1.0);",
            "Named call to `Shape.Rect` is missing `Height`",
        ),
    ] {
        let errors = check_errors(&program(body));
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_INVALID_NAMED_ARGUMENT);
        assert_eq!(errors[0].message, message);
    }
}

#[test]
fn enum_patterns_accept_named_fields() {
    check_ok(&program(
        r#"
  const S: Shape := Shape.Rect(1.0, 2.0);
  case S of
    when Shape.Rect(Width := const W, Height := const H): Console.WriteLn(W + H);
    when Shape.Circle(_): Console.WriteLn(0);
  end case;
"#,
    ));
}

#[test]
fn callable_labels_come_from_the_visible_signature() {
    check_ok(&program(
        r#"
      const F: function(A: integer; B: integer): integer := Sub;
      const N := F(B := 1, A := 9);
      const G := Sub;
      const M := G(Right := 1, Left := 9);
      const R: result of (integer, string) := Ok(Value := Math.Abs(N := -1));
      const O := Some(value := 3);
    "#,
    ));
    for call in ["F(Left := 1, Right := 2)", "F(A := 1, a := 2)", "F(B := 2)"] {
        let errors = check_errors(&program(&format!(
            "const F: function(A: integer; B: integer): integer := Sub; const N := {call};"
        )));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_INVALID_NAMED_ARGUMENT),
            "{call}: {errors:#?}"
        );
    }
    for definition in [
        "function(A: integer; a: integer): integer",
        "procedure(A: integer; a: integer)",
    ] {
        let errors = check_errors(&format!(
            "program T; type Duplicate = {definition}; begin null; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION),
            "{errors:#?}"
        );
    }
}

#[test]
fn named_payloads_and_pattern_fields_require_complete_unique_labels() {
    for (body, code) in [
        ("const R := Some(Wrong := 1);", SEMA_INVALID_NAMED_ARGUMENT),
        (
            "const R: result of (integer, string) := Error(Message := 'x');",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const R: result of (integer, string) := Ok(Wrong := 1);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S := Some(1); const B := S is Some(Wrong := _);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S := Some(1); const B := S is Some(Value := _, value := _);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S: Shape := Shape.Circle(1.0); const B := S is Shape.Rect(Width := _);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S: Shape := Shape.Circle(1.0); const B := S is Shape.Rect(Width := _, width := _);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S: Shape := Shape.Circle(1.0); const B := S is Shape.Rect(Width := _, Wrong := _);",
            SEMA_INVALID_NAMED_ARGUMENT,
        ),
        (
            "const S := Some(1); const B := S is Some(Value := const N);",
            fpas_diagnostics::codes::SEMA_IS_PATTERN_BINDING,
        ),
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|error| error.code == code),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn polymorphic_intrinsic_values_specialize_from_the_receiving_signature() {
    check_ok(&program(
        "const F: function(N: integer): integer := Math.Abs; const A := F(N := -1);",
    ));
}
