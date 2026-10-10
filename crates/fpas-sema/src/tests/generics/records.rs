//! Observable generic-record typing, inference, and default contracts.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH, SEMA_UNSAFE_DISCARD};

const RECORDS: &str = "
type Box of T = record Value: T; end record;
type Pair of (K: Comparable, V) = record Key: K; Value: V; end record;
type Optional of T = record Value: Option of T := None; end record;
type Duo of T = record First: T; Second: T; end record;
";

#[test]
fn fields_annotations_aliases_and_routine_context_infer_records() {
    check_ok(&format!(
        "program T; {RECORDS}
type IntBox = Box of integer;
type Entry of (K, V) = record Key: K; Value: Option of V := None; end record;
function Empty(): Optional of string; begin return Optional(); end function;
function Make<T>(Value: T): Box of T; begin return Box(Value := Value); end function;
procedure Accept(Value: Optional of integer); begin end procedure;
procedure AcceptGeneric<T>(Empty: Optional of T; Evidence: T); begin end procedure;
begin
  const A: Pair of (integer, string) := Pair(Key := 1, Value := 'one');
  const B: Pair of (integer, string) := Pair(Value := 'one', Key := 1);
  const C: Box of array of integer := Box(Value := [1, 2]);
  const D: Box of Box of integer := Box(Value := Box(Value := 3));
  const E: IntBox := IntBox(Value := 4);
  const Partial: Entry of (integer, string) := Entry(Key := 1);
  const F: Box of integer := Make(5);
  var O: Optional of string := Optional();
  O := Optional(Value := None);
  Accept(Optional());
  AcceptGeneric(Optional(), 1);
  AcceptGeneric(Evidence := 1, Empty := Optional());
  const Os: array of Optional of integer := [Optional()];
  const Nested: Option of Optional of integer := Some(Optional());
  const Decision: Optional of integer := if true then Optional() else Optional() end if;
  discard A = B;
end."
    ));
}

#[test]
fn conflicting_evidence_and_incompatible_instances_are_rejected() {
    for statement in [
        "const X: Duo of integer := Duo(First := 1, Second := 'a');",
        "const X: Duo of integer := Duo(Second := 'a', First := 1);",
        "const X: Box of string := Box(Value := 1);",
        "const X: Duo of real := Duo(First := 1, Second := 2.0);",
        "const A: Box of integer := Box(Value := 1); const B: Box of string := A;",
        "const X: Box := Box(Value := 1);",
        "const X: Box of (integer, string) := Box(Value := 1);",
    ] {
        let errors = check_errors(&format!("program T; {RECORDS} begin {statement} end."));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:?}"
        );
    }
}

#[test]
fn type_applications_require_generic_record_declarations() {
    for declaration in [
        "type Plain = integer; begin const X: Plain of integer := 0; end.",
        "type Fixed = record Value: integer; end record; begin const X: Fixed of integer := Fixed(Value := 0); end.",
        "type Shade = enum Dark; end enum; begin const X: Shade of integer := Shade.Dark; end.",
        "type Box of T = record Value: T; end record; type IntBox = Box of integer; begin const X: IntBox of string := IntBox(Value := 0); end.",
    ] {
        let errors = check_errors(&format!("program T; {declaration}"));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
            "{errors:?}"
        );
    }
}

#[test]
fn omitted_defaults_and_empty_payloads_do_not_infer_arguments() {
    for value in ["Optional()", "Optional(Value := None)", "Box(Value := [])"] {
        let errors = check_errors(&format!("program T; {RECORDS} begin discard {value}; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Cannot infer")),
            "{errors:?}"
        );
    }
    let errors = check_errors(&format!(
        "program T; {RECORDS} procedure Require<T>(Value: Optional of T); begin end procedure; begin Require(Optional()); end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Cannot infer")),
        "{errors:?}"
    );
    check_ok(&format!(
        "program T; {RECORDS} begin discard Duo(First := None, Second := Some(1)); discard Duo(Second := Some(1), First := None); end."
    ));
    check_ok(&format!(
        "program T; {RECORDS} begin discard Duo(First := Optional(), Second := Optional(Value := Some(1))); discard Duo(Second := Optional(Value := Some(1)), First := Optional()); end."
    ));
}

#[test]
fn generic_defaults_are_checked_for_every_allowed_argument() {
    check_ok(
        "program T; type Optional of T = record Value: Option of T := None; end record; type Counter of T: Numeric = record Value: Option of T := None; Step: integer := 1; end record; begin const C: Counter of integer := Counter(); end.",
    );
    let errors = check_errors(
        "program T; type Bad of T = record Value: T := 0; end record; begin const B: Bad of integer := Bad(); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("default value")),
        "{errors:?}"
    );
}

#[test]
fn constraints_apply_to_annotations_inference_and_enclosing_parameters() {
    for source in [
        "type N of T: Numeric = record Value: T; end record; begin const X: N of string := N(Value := 'a'); end.",
        "type N of T: Numeric = record Value: T; end record; begin discard N(Value := 'a'); end.",
        "type N of T: Numeric = record Value: T; end record; function Make<T>(Value: T): N of T; begin return N(Value := Value); end function; begin end.",
    ] {
        let errors = check_errors(&format!("program T; {source}"));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{errors:?}"
        );
    }
    check_ok(
        "program T; type N of T: Comparable = record Value: T; end record; function Make<T: Numeric>(Value: T): N of T; begin return N(Value := Value); end function; begin const X: N of integer := Make(1); end.",
    );
}

#[test]
fn recursive_records_forward_parameters_and_still_require_finite_values() {
    check_ok(
        "program T; type Node of T = record Value: T; Next: Option of Node of T := None; end record; begin const N: Node of integer := Node(Value := 1); end.",
    );
    check_ok(
        "program T; type A of T = record Value: T; Next: Option of B of T := None; end record; type B of U = record Value: U; Next: Option of A of U := None; end record; begin const N: A of integer := A(Value := 1); end.",
    );
    for arguments in ["string", "array of T", "Option of T"] {
        let errors = check_errors(&format!(
            "program T; type Node of T = record Next: Option of Node of {arguments} := None; end record; begin end."
        ));
        assert!(
            errors.iter().any(|error| error.message.contains("forward")),
            "{errors:?}"
        );
    }
    let errors =
        check_errors("program T; type Node of T = record Next: Node of T; end record; begin end.");
    assert!(
        errors.iter().any(|error| error.message.contains("finite")),
        "{errors:?}"
    );
    let errors = check_errors(
        "program T; type Box of T = record Value: T; end record; type Node = record Next: Box of Node; end record; begin end.",
    );
    assert!(
        errors.iter().any(|error| error.message.contains("finite")),
        "{errors:?}"
    );
    let errors = check_errors(
        "program T; type Node of (K, V) = record Next: Option of Node of (V, K) := None; end record; begin end.",
    );
    assert!(
        errors.iter().any(|error| error.message.contains("forward")),
        "{errors:?}"
    );
}

#[test]
fn generic_record_methods_require_the_declaring_receiver_arguments() {
    let errors = check_errors(
        "program T; type Box of T = record Value: T; procedure Visit(Self: Box of string); begin end procedure; end record; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Self: Box of T")),
        "{errors:?}"
    );
}

#[test]
fn method_inference_preserves_enclosing_parameters_with_matching_names() {
    let declarations = "type Box of T = record Value: T; procedure Check(Self: Box of T; Other: T); begin end procedure; function Map<R>(Self: Box of T; F: function(Value: T): R): R; begin return F(Self.Value); end function; end record; function ReadInteger(Value: integer): string; begin return 'integer'; end function;";
    for parameter in ["X", "R"] {
        for statement in [
            "Value.Check(1);",
            "const Text: string := Value.Map(ReadInteger);",
        ] {
            let errors = check_errors(&format!(
                "program T; {declarations} procedure Use<{parameter}>(Value: Box of {parameter}); begin {statement} end procedure; begin end."
            ));
            assert!(
                errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
                "{errors:?}"
            );
        }
    }
    check_ok(&format!(
        "program T; {declarations} procedure Use<R>(Value: Box of R; Evidence: R); function Read(Other: R): string; begin return 'value'; end function; begin Value.Check(Evidence); const Text: string := Value.Map(Read); end procedure; begin end."
    ));
    check_ok(&format!(
        "program T; {declarations} function GenericRead<S>(Other: S): string; begin return 'value'; end function; procedure Use<R>(Value: Box of R); begin const Text: string := Value.Map(GenericRead); end procedure; begin end."
    ));
}

#[test]
fn nested_instances_do_not_hide_unsupported_equality_or_task_payloads() {
    let errors = check_errors(
        "program T; type Box of T = record Value: T; end record; begin const A: Box of Box of array of integer := Box(Value := Box(Value := [1])); discard A = A; end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:?}"
    );
    let errors = check_errors(
        "program T; type Box of T = record Value: T; end record; function Work(): integer; begin return 1; end function; begin discard Box(Value := Box(Value := go Work())); end.",
    );
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD),
        "{errors:?}"
    );
}
