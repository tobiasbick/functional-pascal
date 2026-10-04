use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{CompilationUnit, Unit, parse_compilation_unit};
use fpas_unit::interface::UnitInterface;

use super::{check_errors, check_ok};
use crate::analyze_unit;

fn parse_unit(source: &str) -> Unit {
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "unexpected parse errors: {errors:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must parse as a unit");
    };
    unit
}

fn interface_for(source: &str) -> UnitInterface {
    let analysis = analyze_unit(&parse_unit(source), &[]).expect("unit analysis must succeed");
    assert!(
        analysis.metadata.errors.is_empty(),
        "unexpected sema errors: {:#?}",
        analysis.metadata.errors
    );
    analysis.interface.expect("valid unit interface")
}

#[test]
fn same_record_declaration_and_alias_are_compatible() {
    check_ok(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

type PointAlias = Point;

begin
  var PointValue: Point := Point(X := 1, Y := 2);
  var SameType: Point := PointValue;
  var AliasValue: PointAlias := PointValue;
  var ContextualLiteral: PointAlias := Point(X := 3, Y := 4);
end program;
"#,
    );
}

#[test]
fn distinct_public_record_declarations_are_incompatible_despite_equal_fields() {
    let errors = check_errors(
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

type Size = record
  X: integer;
  Y: integer;
end record;

begin
  var SizeValue: Size := Size(X := 1, Y := 2);
  var PointValue: Point := SizeValue;
end program;
"#,
    );

    assert!(
        errors.iter().any(|error| {
            error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("expected `Point`, found `Size`")
        }),
        "expected nominal record mismatch, got: {errors:#?}"
    );
}

#[test]
fn distinct_private_records_are_incompatible_inside_their_owner_unit() {
    let unit = parse_unit(
        r#"unit Demo.PrivateRecords;  type Left = record Value: integer; end record;  type Right = record Value: integer; end record; function Convert(Value: Right): Left; begin return Value; end function;
end unit;
"#,
    );
    let analysis = analyze_unit(&unit, &[]).expect("unit analysis must succeed");

    assert!(
        analysis.metadata.errors.iter().any(|error| {
            error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("expected `Left`, found `Right`")
        }),
        "expected private nominal record mismatch, got: {:#?}",
        analysis.metadata.errors
    );
}

#[test]
fn imported_records_use_their_qualified_declaration_identity() {
    let interfaces = [
        interface_for(
            r#"unit Demo.First;   public type Value = record public Number: integer; end record;
end unit;
"#,
        ),
        interface_for(
            r#"unit Demo.Second;   public type Value = record public Number: integer; end record;
end unit;
"#,
        ),
    ];
    let consumer = parse_unit(
        r#"unit Demo.Consumer;  uses Demo.First as First; uses Demo.Second as Second; public function Keep(Value: First.Value): First.Value; begin return Value; end function; public function Reject(Value: Second.Value): First.Value; begin return Value; end function;
end unit;
"#,
    );
    let analysis = analyze_unit(&consumer, &interfaces).expect("consumer analysis must succeed");

    assert_eq!(
        analysis.metadata.errors.len(),
        1,
        "{:#?}",
        analysis.metadata.errors
    );
    assert_eq!(analysis.metadata.errors[0].code, SEMA_TYPE_MISMATCH);
    assert!(
        analysis.metadata.errors[0]
            .message
            .contains("expected `demo.first.value`, found `demo.second.value`"),
        "unexpected mismatch: {:#?}",
        analysis.metadata.errors
    );
}

#[test]
fn anonymous_generic_binding_does_not_bridge_distinct_named_records() {
    let errors = check_errors(
        r#"program T;

type Left = record
  Value: integer;
end record;

type Right = record
  Value: integer;
end record;

function Pick of (TValue)(A: TValue; B: TValue; C: TValue): TValue;
begin
  return A;
end function;

begin
  var LeftValue: Left := Left(Value := 1);
  var RightValue: Right := Right(Value := 2);
  var ResultValue: Left := Pick(Left(Value := 0), LeftValue, RightValue);
end program;
"#,
    );

    assert!(
        errors.iter().any(|error| {
            error.code == SEMA_TYPE_MISMATCH
                && error
                    .message
                    .contains("inferred as `Left`, but was also used with `Right`")
        }),
        "expected generic nominal record mismatch, got: {errors:#?}"
    );
}
