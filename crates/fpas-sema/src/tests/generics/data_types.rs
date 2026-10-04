//! Generic nominal type applications and recursion.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH};

#[test]
fn generic_records_resolve_fields_and_forward_routine_arguments() {
    check_ok(
        "program Main;\n\ntype Box of (T) = record\n  Value: T;\nend record;\n\nfunction ReadBox of (T)(Value: Box of (T)): T;\nbegin\n  return Value.Value;\nend function;\n\nfunction Forward of (U)(Value: Box of (U)): U;\nbegin\n  return ReadBox(Value);\nend function;\n\nvar Item: Box of (integer) := Box(Value := 42);\n\nbegin\n  discard Forward(Item);\nend program;\n",
    );
}

#[test]
fn applications_resolve_independently_of_declaration_order() {
    check_ok(
        "program Main;
        type IntegerBox = Box of (integer);
        type Wrapper of (U) = record Item: Box of (U); end record;
        type Box of (T) = record Value: T; end record;
        function ReadValue(Value: Wrapper of (integer)): integer;
        begin return Value.Item.Value; end function;
        begin null; end program;",
    );
}

#[test]
fn generic_records_preserve_nominal_argument_identity() {
    let errors = check_errors(
        "program Main;\n\ntype Marker of (T) = record\nend record;\n\nvar First: Marker of (integer) := Marker();\nvar Second: Marker of (string) := First;\n\nbegin\n  null;\nend program;\n",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn generic_enums_preserve_nominal_argument_identity() {
    let errors = check_errors(
        "program Main;
        type Marker of (T) = enum Present; Missing; end enum;
        function Same(First: Marker of (integer); Second: Marker of (string)): boolean;
        begin return First = Second; end function;
        begin null; end program;",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}

#[test]
fn type_arguments_enforce_constraints_for_concrete_and_forwarded_types() {
    for argument in ["string", "U"] {
        let heading = if argument == "U" {
            "type Forward of (U) = record Item: NumericBox of (U); end record;"
        } else {
            "type Invalid = NumericBox of (string);"
        };
        let errors = check_errors(&format!(
            "program Main;
            type NumericBox of (T: Numeric) = record Value: T; end record;
            {heading} begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{errors:#?}"
        );
    }
    check_ok(
        "program Main;
        type NumericBox of (T: Numeric) = record Value: T; end record;
        type Forward of (U: Numeric) = record Item: NumericBox of (U); end record;
        begin null; end program;",
    );
}

#[test]
fn type_applications_require_exact_arity_and_cannot_reapply_aliases() {
    for target in [
        "Box",
        "Box of (integer, string)",
        "integer of (string)",
        "IntegerBox of (integer)",
    ] {
        let errors = check_errors(&format!(
            "program Main;
            type Box of (T) = record Value: T; end record;
            type IntegerBox = Box of (integer);
            type Invalid = {target}; begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH
                    && error.message.contains("type argument")),
            "{target}: {errors:#?}"
        );
    }
}

#[test]
fn recursive_generic_fields_resolve_at_each_selection() {
    check_ok(
        "program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (T)); end record;
        function Descendant(Value: Node of (integer)): integer;
        begin return Value.Children[0].Children[0].Value; end function;
        begin null; end program;",
    );
}

#[test]
fn polymorphic_recursion_keeps_transformed_arguments() {
    check_ok(
        "program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (array of (T))); end record;
        function Descendant(Value: Node of (integer)): array of (integer);
        begin return Value.Children[0].Value; end function;
        begin null; end program;",
    );
}

#[test]
fn recursive_component_checks_terminate_and_inspect_changed_arguments() {
    check_ok(
        "program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (array of (T))); end record;
        function Same(First: Node of (integer); Second: Node of (integer)): boolean;
        begin return First = Second; end function;
        procedure Ignore(Value: Node of (integer));
        begin discard Value; end procedure;
        begin null; end program;",
    );
    let errors = check_errors("program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (function(): integer)); end record;
        function Same(First: Node of (integer); Second: Node of (integer)): boolean;
        begin return First = Second; end function;
        begin null; end program;");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Equality requires")),
        "{errors:#?}"
    );
    let errors = check_errors("program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (task of (integer))); end record;
        procedure Ignore(Value: Node of (integer)); begin discard Value; end procedure;
        begin null; end program;");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Cannot discard a task")),
        "{errors:#?}"
    );
}

#[test]
fn finite_values_allow_collection_and_enum_base_cases() {
    check_ok(
        "program Main;
        type Tree of (T) = enum Leaf(Value: T); Branch(Children: array of (Tree of (T))); end enum;
        type List of (T) = enum Empty; Link(Value: T; Tail: List of (T)); end enum;
        type MutualRecord = record Item: MutualEnum; end record;
        type MutualEnum = enum Stop; Resume(Value: MutualRecord); end enum;
        type OptionalRecord = record Next: Option of (OptionalRecord); end record;
        type Wrapper = record First: MutualEnum; Second: MutualRecord; end record;
        type Box of (T) = record Value: T; end record;
        begin null; end program;",
    );
}

#[test]
fn mandatory_recursive_types_have_no_finite_value() {
    for declarations in [
        "type Loop = record Next: Loop; end record;",
        "type Loop of (T) = record Next: Loop of (array of (T)); end record;",
        "type Loop = enum Next(Value: Loop); end enum;",
        "type First = record Next: Second; end record; type Second = record Next: First; end record;",
        "type Loop = record Next: Result of (Loop, Loop); end record;",
    ] {
        let errors = check_errors(&format!(
            "program Main; {declarations} begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("no finite representable value")),
            "{declarations}: {errors:#?}"
        );
    }
}

#[test]
fn recursive_generic_aliases_remain_transparent_alias_cycles() {
    for argument in ["Loop", "Option of (Loop)"] {
        let errors = check_errors(&format!(
            "program Main;
            type Box of (T) = record Value: T; end record;
            type Loop = Box of ({argument}); begin null; end program;"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Cyclic type alias")),
            "{errors:#?}"
        );
    }
}

#[test]
fn nested_instances_of_one_generic_type_keep_component_capabilities() {
    let errors = check_errors("program Main;
        type Box of (T) = record Value: T; end record;
        procedure Ignore(Value: Box of (Box of (task of (integer)))); begin discard Value; end procedure;
        begin null; end program;");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Cannot discard a task")),
        "{errors:#?}"
    );
    let errors = check_errors("program Main;
        type Box of (T) = record Value: T; end record;
        function Same(First: Box of (Box of (function(): integer)); Second: Box of (Box of (function(): integer))): boolean;
        begin return First = Second; end function;
        begin null; end program;");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Equality requires")),
        "{errors:#?}"
    );
}

#[test]
fn root_headers_cannot_inherit_another_declarations_parameters() {
    let errors = check_errors(
        "program Main;
        type Outer of (T) = record Inner: Broken; end record;
        type Broken = T;
        begin null; end program;",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Unknown type `T`")),
        "{errors:#?}"
    );
}
