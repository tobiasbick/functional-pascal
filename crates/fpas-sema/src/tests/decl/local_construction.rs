//! Constructor identity and complete type evidence for unannotated local bindings.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TYPE_ANNOTATION_REQUIRED, SEMA_TYPE_MISMATCH};

const TYPES: &str = "program T;
type Point = record X: integer; Y: integer := 2; end record;
type Position = Point;
type Empty = record end record;
type Box of T = record Value: T; end record;
type Defaults of T = record Value: Option of T := None; end record;
type StringBox = Box of string;
type Color = enum Red; Blue; end enum;
type Shape = enum Circle(Radius: real); Origin; end enum;
type Lookup of T = enum Found(Value: T); Missing; end enum;
type Choice of (L, R) = enum Left(Value: L); Right(Value: R); end enum;
type TextLookup = Lookup of string;
function MakePoint(): Point; begin return Point(X := 1); end function;
function MakeShape(): Shape; begin return Shape.Origin; end function;";

#[test]
fn complete_local_record_and_enum_constructions_infer_their_nominal_types() {
    check_ok(&format!(
        "{TYPES} begin\n        const P := (Point(X := 1)); const Alias := Position(X := 2);\n        const Zero := Empty(); const Number := Box(Value := 1);\n        const Fixed := StringBox(Value := 'fixed');\n        const Nested := Box(Value := Box(Value := 'nested'));\n        const Defaulted: Defaults of integer := Defaults();\n        const Red := Color.Red; const Short := Color.Blue;\n        var S := Shape.Circle(Radius := 2.0); S := Shape.Origin;\n        const Found := Lookup.Found(Value := Box(Value := 3));\n        const Missing := TextLookup.Missing; const Wrapped := Some(Lookup.Found('text'));\n        const Keep: Point := P; const KeepAlias: Point := Alias; const KeepEmpty: Empty := Zero;\n        const KeepNumber: Box of integer := Number; const KeepFixed: Box of string := Fixed;\n        const KeepNested: Box of Box of string := Nested;\n        const KeepRed: Color := Red; const KeepShort: Color := Short;\n        const KeepFound: Lookup of Box of integer := Found;\n        const KeepMissing: Lookup of string := Missing;\n        const KeepWrapped: Option of Lookup of string := Wrapped;\n        end."
    ));
}

#[test]
fn factories_and_constructor_results_followed_by_access_infer() {
    for initializer in [
        "MakePoint()",
        "MakeShape()",
        "Point(X := 1).X",
        "Point(X := 1) with X := 2; end with",
    ] {
        check_ok(&format!("{TYPES} begin const Value := {initializer}; end."));
    }
}

#[test]
fn callable_shadowing_a_record_or_variant_is_an_ordinary_call() {
    for (name, result, value) in [
        ("Point", "Point", "Point(X := 1)"),
        ("Found", "Lookup of integer", "Lookup.Found(1)"),
    ] {
        check_ok(&format!("{TYPES} begin
            const Factory: function(): {result} := function(): {result} begin return {value}; end function;
            begin const {name}: function(): {result} := Factory; const Value := {name}(); end;
            end."));
    }
}

#[test]
fn underdetermined_constructors_require_context_instead_of_guessing_arguments() {
    for initializer in [
        "Box(Value := [])",
        "Defaults()",
        "Lookup.Missing",
        "Choice.Left(1)",
    ] {
        let errors = check_errors(&format!("{TYPES} begin const Value := {initializer}; end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("Add a type annotation"))),
            "{initializer}: {errors:#?}"
        );
    }
    for initializer in [
        "None",
        "Ok(1)",
        "Error('failure')",
        "Some([])",
        "Some(None)",
    ] {
        let errors = check_errors(&format!("{TYPES} begin const Value := {initializer}; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
            "{initializer}: {errors:#?}"
        );
    }
    check_ok(&format!("{TYPES} begin
        const Empty: Box of array of integer := Box(Value := []);
        const Defaulted: Defaults of integer := Defaults();
        const Missing: Lookup of string := Lookup.Missing;
        const Left: Choice of (integer, string) := Choice.Left(1);
        const Absent: Option of integer := None; const Success: Result of (integer, string) := Ok(1);
        end."));
}

#[test]
fn inferred_constructions_preserve_type_identity_and_argument_diagnostics() {
    for statement in [
        "var Value := Box(Value := 1); Value := Box(Value := 'wrong');",
        "var Value := Color.Red; Value := Shape.Origin;",
        "const Value := Point(X := 'wrong');",
        "const Value := Shape.Circle('wrong');",
    ] {
        let errors = check_errors(&format!("{TYPES} begin {statement} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{statement}: {errors:#?}"
        );
        assert!(
            !errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_ANNOTATION_REQUIRED),
            "{statement}: {errors:#?}"
        );
    }
}

#[test]
fn generic_callable_constructor_payloads_require_a_concrete_signature() {
    let prefix =
        format!("{TYPES} function Identity<T>(Value: T): T; begin return Value; end function;");
    for initializer in ["Box(Value := Identity)", "Lookup.Found(Identity)"] {
        let errors = check_errors(&format!(
            "{prefix} begin const Value := {initializer}; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH
                    && error.message.contains("Cannot infer")),
            "{initializer}: {errors:#?}"
        );
    }
    check_ok(&format!(
        "{prefix} begin
        const Boxed: Box of function(Value: integer): integer := Box(Value := Identity);
        const Found: Lookup of function(Value: integer): integer := Lookup.Found(Identity);
        end."
    ));
}
