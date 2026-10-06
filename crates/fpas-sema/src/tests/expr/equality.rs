//! `=` and `<>` accept records and payload enums whose fields all compare, and nothing else.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const TYPES: &str = "program T;
type
  Point = record X: integer; Y: real; end record;
  Named = record Point: Point; Name: string; Tag: option of Point; end record;
  Shape = enum Circle(Center: Point; Radius: integer); Dot; end enum;
  Bag = record Items: array of integer; end record;
  Holder = enum Full(Values: array of integer); Empty; end enum;
  Callback = record Run: function(): integer; end record;
  Tree = enum Leaf(Value: integer); Node(Left: option of Tree; Right: option of Tree); end enum;
";

fn errors_for(declarations: &str, condition: &str) -> Vec<crate::SemaError> {
    check_errors(&format!(
        "{TYPES}{declarations}
begin
  var Same: boolean := {condition};
end."
    ))
}

#[test]
fn records_and_payload_enums_with_comparable_fields_support_equality() {
    for (declarations, condition) in [
        ("var A: Point := record X := 1; Y := 2.0; end;", "A = A"),
        (
            "var A: Named := record Point := record X := 1; Y := 2.0; end; Name := 'a'; Tag := None; end;",
            "A <> A",
        ),
        (
            "var S: Shape := Shape.Dot;",
            "S = Shape.Circle(record X := 1; Y := 1.0; end, 2)",
        ),
        (
            "var T: Tree := Tree.Leaf(1);",
            "T = Tree.Node(None, Some(Tree.Leaf(2)))",
        ),
    ] {
        check_ok(&format!(
            "{TYPES}{declarations}
begin
  var Same: boolean := {condition};
end."
        ));
    }
}

#[test]
fn aggregates_with_non_comparable_fields_reject_equality() {
    for (declarations, condition) in [
        ("var B: Bag := record Items := []; end;", "B = B"),
        ("var H: Holder := Holder.Empty;", "H = Holder.Empty"),
        (
            "var C: Callback := record Run := function(): integer begin return 1; end; end;",
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
        "var A: Point := record X := 1; Y := 2.0; end; var S: Shape := Shape.Dot;",
        "A = S",
    );
    assert_eq!(different.len(), 1, "{different:#?}");
    let ordered = errors_for("var A: Point := record X := 1; Y := 2.0; end;", "A < A");
    assert_eq!(ordered.len(), 1, "{ordered:#?}");
}
