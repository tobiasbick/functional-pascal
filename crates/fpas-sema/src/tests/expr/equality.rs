//! `=` and `<>` accept options, results, records, and payload enums whose payloads and fields
//! all compare, and nothing else.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const TYPES: &str = "program T;
type
  Point = record X: integer; Y: real; end record;
  type Named = record Point: Point; Name: string; Tag: option of Point; end record;
  type Shape = enum Circle(Center: Point; Radius: integer); Dot; end enum;
  type Bag = record Items: array of integer; end record;
  type Holder = enum Full(Values: array of integer); Empty; end enum;
  type Callback = record Run: function(): integer; end record;
  type Tree = enum Leaf(Value: integer); Node(Left: option of Tree; Right: option of Tree); end enum;
";

fn errors_for(declarations: &str, condition: &str) -> Vec<crate::SemaError> {
    check_errors(&format!(
        "{TYPES}{declarations}
begin
  const Same: boolean := {condition};
end."
    ))
}

#[test]
fn records_and_payload_enums_with_comparable_fields_support_equality() {
    for (declarations, condition) in [
        ("const A: Point := Point( X := 1, Y := 2.0 );", "A = A"),
        (
            "const A: Named := Named( Point := Point( X := 1, Y := 2.0 ), Name := 'a', Tag := None );",
            "A <> A",
        ),
        (
            "const S: Shape := Shape.Dot;",
            "S = Shape.Circle(Point( X := 1, Y := 1.0 ), 2)",
        ),
        (
            "const T: Tree := Tree.Leaf(1);",
            "T = Tree.Node(None, Some(Tree.Leaf(2)))",
        ),
    ] {
        check_ok(&format!(
            "{TYPES}{declarations}
begin
  const Same: boolean := {condition};
end."
        ));
    }
}

#[test]
fn aggregates_with_non_comparable_fields_reject_equality() {
    for (declarations, condition) in [
        ("const B: Bag := Bag( Items := [] );", "B = B"),
        ("const H: Holder := Holder.Empty;", "H = Holder.Empty"),
        (
            "const C: Callback := Callback( Run := function(): integer begin return 1; end function );",
            "C <> C",
        ),
    ] {
        let errors = errors_for(declarations, condition);
        assert_eq!(errors.len(), 1, "{condition}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{condition}");
        assert!(
            errors[0].message.contains("Equality requires"),
            "{condition}: {errors:#?}"
        );
    }
}

#[test]
fn records_of_different_types_and_ordering_stay_rejected() {
    let different = errors_for(
        "const A: Point := Point( X := 1, Y := 2.0 ); const S: Shape := Shape.Dot;",
        "A = S",
    );
    assert_eq!(different.len(), 1, "{different:#?}");
    let ordered = errors_for("const A: Point := Point( X := 1, Y := 2.0 );", "A < A");
    assert_eq!(ordered.len(), 1, "{ordered:#?}");
}

#[test]
fn options_and_results_compare_only_when_their_payloads_compare() {
    for (declarations, condition) in [
        ("const O: option of integer := Some(1);", "O = None"),
        (
            "const O: option of Point := None;",
            "O <> Some(Point( X := 1, Y := 2.0 ))",
        ),
        (
            "const R: result of integer, string := Ok(1);",
            "R = Error('x')",
        ),
    ] {
        check_ok(&format!(
            "{TYPES}{declarations}
begin
  const Same: boolean := {condition};
end."
        ));
    }
    for (declarations, condition) in [
        ("const O: option of array of integer := Some([1]);", "O = O"),
        ("const O: option of Bag := None;", "O = None"),
        (
            "const R: result of integer, array of string := Ok(1);",
            "R = R",
        ),
        (
            "const O: option of function(): integer := None;",
            "O = None",
        ),
    ] {
        let errors = errors_for(declarations, condition);
        assert_eq!(errors.len(), 1, "{condition}: {errors:#?}");
        assert!(
            errors[0].message.contains("Equality requires"),
            "{condition}: {errors:#?}"
        );
    }
}

#[test]
fn generic_option_payloads_compare_only_with_a_comparable_constraint() {
    check_ok(
        "program T;
         function IsEmpty<T: Comparable>(Value: option of T): boolean;
         begin return Value = None; end function;
         begin end.",
    );
    let errors = check_errors(
        "program T;
         function IsEmpty<T>(Value: option of T): boolean;
         begin return Value = None; end function;
         begin end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH, "{errors:#?}");
}
