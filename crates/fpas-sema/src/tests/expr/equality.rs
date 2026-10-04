//! Structural equality follows every component of value data.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const TYPES: &str = "program T;\ntype Point = record X: integer; Y: real; end record;\ntype Named = record Point: Point; Name: string; Tag: option of (Point); end record;\ntype Shape = enum Circle(Center: Point; Radius: integer); Dot; end enum;\ntype Bag = record Items: array of (integer); end record;\ntype Holder = enum Full(Values: array of (integer)); Empty; end enum;\ntype Callback = record Run: function(): integer; end record;\ntype Tree = enum Leaf(Value: integer); Node(Left: option of (Tree); Right: option of (Tree)); end enum;\n";

fn errors_for(declarations: &str, condition: &str) -> Vec<crate::SemaError> {
    check_errors(&format!(
        r#"{TYPES}{declarations}
begin
  const Same: boolean := {condition};
end program;"#
    ))
}

#[test]
fn records_and_payload_enums_with_comparable_fields_support_equality() {
    for (declarations, condition) in [
        ("var A: Point := Point(X := 1, Y := 2.0);", "A = A"),
        (
            "var A: Named := Named(Point := Point(X := 1, Y := 2.0), Name := 'a', Tag := Option.None);",
            "A <> A",
        ),
        (
            "var S: Shape := Shape.Dot;",
            "S = Shape.Circle(Point(X := 1, Y := 1.0), 2)",
        ),
        (
            "var T: Tree := Tree.Leaf(1);",
            "T = Tree.Node(Option.None, Option.Some(Tree.Leaf(2)))",
        ),
        ("var B: Bag := Bag(Items := []);", "B = B"),
        ("var H: Holder := Holder.Empty;", "H = Holder.Empty"),
    ] {
        check_ok(&format!(
            r#"{TYPES}{declarations}
begin
  const Same: boolean := {condition};
end program;"#
        ));
    }
}

#[test]
fn aggregates_with_non_comparable_fields_reject_equality() {
    for (declarations, condition) in [(
        r#"const C: Callback := Callback(Run := function(): integer begin return 1; end function);"#,
        "C <> C",
    )] {
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
fn option_result_and_collections_reject_callable_and_resource_components() {
    for (imports, ty, value) in [
        ("", "option of (function(): integer)", "Option.None"),
        ("", "result of (integer, procedure())", "Result.Ok(1)"),
        (
            "",
            "array of (option of (function(): integer))",
            "[Option.None]",
        ),
        (
            "",
            "dict of (string, option of (procedure()))",
            "['a': Option.None]",
        ),
        (
            "uses Std.Net as Net;",
            "option of (Net.Connection)",
            "Option.None",
        ),
        (
            "uses Std.Console as Console;",
            "array of (Console.SavedRegion)",
            "[]",
        ),
        (
            "uses Std.Tasks as Tasks;",
            "option of (Tasks.TaskGroup)",
            "Option.None",
        ),
        ("", "option of (channel of (integer))", "Option.None"),
        ("", "option of (task of (integer))", "Option.None"),
    ] {
        let errors = check_errors(&format!(
            r#"program Main; {imports}
            begin const Value: {ty} := {value}; const Same: boolean := Value = Value; end program;"#
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Equality requires")),
            "{ty}: {errors:#?}"
        );
    }
}

#[test]
fn records_of_different_types_and_ordering_stay_rejected() {
    let different = errors_for(
        "var A: Point := Point(X := 1, Y := 2.0); var S: Shape := Shape.Dot;",
        "A = S",
    );
    assert_eq!(different.len(), 1, "{different:#?}");
    let ordered = errors_for("var A: Point := Point(X := 1, Y := 2.0);", "A < A");
    assert_eq!(ordered.len(), 1, "{ordered:#?}");
}

#[test]
fn empty_constructor_components_accept_concrete_value_context() {
    check_ok(
        "program Main; begin\n           discard Option.None = Option.Some(1); discard Result.Ok(1) <> Result.Error('failed');\n           discard [Option.None, Option.Some(1)] = [Option.Some(1), Option.None];\n           discard ['a': Option.None, 'b': Option.Some(1)] = ['b': Option.Some(1), 'a': Option.None];\n         end program;",
    );
}

#[test]
fn later_literal_elements_cannot_hide_callable_components() {
    for value in [
        "[Option.None, Option.Some(Handler)]",
        "[[Option.None], [Option.Some(Handler)]]",
        "['a': Option.None, 'b': Option.Some(Handler)]",
        "[Box(Value := Option.None), Box(Value := Option.Some(Handler))]",
    ] {
        let errors = check_errors(&format!(
            "program Main; type Box of (T) = record Value: T; end record; procedure Handler(); begin null; end procedure;\n             begin discard {value} = {value}; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Equality requires")),
            "{value}: {errors:#?}"
        );
    }
}

#[test]
fn empty_collections_do_not_allow_callable_searches() {
    for expression in [
        "Handler in []",
        "Option.Some(Handler) in [Option.None]",
        "Arrays.Contains([], Handler)",
        "Arrays.IndexOf([Option.None], Option.Some(Handler))",
    ] {
        let errors = check_errors(&format!(
            "program Main; uses Std.Arrays as Arrays;
             procedure Handler(); begin null; end procedure;
             begin discard {expression}; end program;"
        ));
        assert!(!errors.is_empty(), "{expression}");
        assert!(
            errors.iter().any(|error| error
                .help
                .as_deref()
                .is_some_and(|help| help.contains("callable"))),
            "{expression}: {errors:#?}"
        );
    }
}
