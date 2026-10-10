//! Generic enum inference, nominal applications, recursive payloads, and patterns.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_TYPE_MISMATCH, SEMA_UNSAFE_DISCARD};

const ENUMS: &str = "
type Lookup of T = enum Found(Value: T); Missing; end enum;
type Choice of (L, R) = enum Left(Value: L); Right(Value: R); Neither; end enum;
type Duo of T = enum Pair(First: T; Second: T); Empty; end enum;
type Tree of T = enum Leaf(Value: T); Node(Left: Tree of T; Right: Tree of T); Empty; end enum;
";

#[test]
fn payloads_expected_types_aliases_and_routine_evidence_infer_enums() {
    check_ok(&format!("program T; {ENUMS}
type IntLookup = Lookup of integer;
type Box of T = record Value: T; end record;
function Empty(): Lookup of string; begin return Lookup.Missing; end function;
function Make<T>(Value: T): Lookup of T; begin return Lookup.Found(Value); end function;
procedure Accept(Value: Lookup of integer); begin end procedure;
procedure GenericAccept<T>(Value: Lookup of T; Evidence: T); begin end procedure;
procedure Consume<T>(Value: Duo of Lookup of T); begin end procedure;
begin
  const A: Lookup of integer := Lookup.Found(1);
  const B: Lookup of string := Make('one');
  const C: Choice of (integer, string) := Choice.Left(1);
  const D: Choice of (integer, string) := Choice.Right('one');
  const E: IntLookup := IntLookup.Missing;
  const F: Lookup of array of integer := Lookup.Found([1, 2]);
  const G: Lookup of Box of integer := Lookup.Found(Box(Value := 3));
  const H: Box of Lookup of string := Box(Value := Lookup.Missing);
  var I: Lookup of integer := Lookup.Missing;
  I := Lookup.Found(2);
  Accept(Lookup.Missing);
  GenericAccept(Lookup.Missing, 1);
  GenericAccept(Evidence := 1, Value := Lookup.Missing);
  const Values: array of Lookup of integer := [Lookup.Missing, Lookup.Found(3)];
  const O: Option of Lookup of integer := Some(Lookup.Missing);
  const Decision: Lookup of integer := if true then Lookup.Missing else Lookup.Found(1) end if;
  const Joint: Duo of Lookup of integer := Duo.Pair(First := Lookup.Missing, Second := Lookup.Found(1));
  const Reversed: Duo of Lookup of integer := Duo.Pair(Second := Lookup.Found(1), First := Lookup.Missing);
  Consume(Duo.Pair(First := Lookup.Missing, Second := Lookup.Found(1)));
  Consume(Duo.Pair(Second := Lookup.Found(1), First := Lookup.Missing));
end."));
}

#[test]
fn conflicts_undetermined_arguments_and_bare_or_wrong_arity_types_are_rejected() {
    for (statement, message) in [
        ("discard Lookup.Missing;", "Cannot infer"),
        ("discard Choice.Left(1);", "Cannot infer"),
        ("discard Lookup.Found([]);", "Cannot infer"),
        ("discard Duo.Pair(First := 1, Second := 'a');", "inferred"),
        ("discard Duo.Pair(Second := 'a', First := 1);", "inferred"),
        ("discard Duo.Pair(1, 2.0);", "inferred"),
        ("const X: Lookup of string := Lookup.Found(1);", "inferred"),
        (
            "const A: Lookup of integer := Lookup.Found(1); const B: Lookup of string := A;",
            "mismatch",
        ),
        (
            "const X: Lookup := Lookup.Found(1);",
            "requires type arguments",
        ),
        (
            "const X: Lookup of (integer, string) := Lookup.Found(1);",
            "expects 1 type",
        ),
        (
            "const X: Choice of integer := Choice.Left(1);",
            "expects 2 type",
        ),
    ] {
        let errors = check_errors(&format!("program T; {ENUMS} begin {statement} end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains(message)),
            "{statement}: {errors:?}"
        );
    }
}

#[test]
fn constraints_apply_to_payload_inference_and_annotations() {
    check_ok(
        "program T; type Key of T: Comparable = enum Value(Item: T); Empty; end enum; function Make<T: Numeric>(Value: T): Key of T; begin return Key.Value(Value); end function; begin const K: Key of integer := Make(1); end.",
    );
    for source in [
        "begin discard Key.Value([1]); end.",
        "begin const X: Key of array of integer := Key.Empty; end.",
        "function Make<T>(Value: T): Key of T; begin return Key.Value(Value); end function; begin end.",
    ] {
        let errors = check_errors(&format!(
            "program T; type Key of T: Comparable = enum Value(Item: T); Empty; end enum; {source}"
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_CONSTRAINT_VIOLATION),
            "{errors:?}"
        );
    }
}

#[test]
fn nested_patterns_bind_instantiated_payloads_and_check_coverage() {
    check_ok(&format!("program T; {ENUMS}
function Read(Value: Lookup of Lookup of integer): integer;
begin return case Value of when Lookup.Found(Lookup.Found(const N)): N + 1; when Lookup.Found(Lookup.Missing): 0; when Lookup.Missing: -1; end case; end function;
function First<T>(Value: Tree of T): Option of T;
begin return case Value of when Tree.Leaf(const Item): Some(Item); when Tree.Node(const L, _): First(L); when Tree.Empty: None; end case; end function;
begin const X: integer := Read(Lookup.Found(Lookup.Found(2))); end."));
    for (source, message) in [
        (
            "function Read(Value: Lookup of Lookup of integer): integer; begin return case Value of when Lookup.Found(Lookup.Found(const N)): N; when Lookup.Missing: 0; end case; end function;",
            "Missing",
        ),
        (
            "function Read(Value: Lookup of integer): integer; begin return case Value of when Lookup.Found(const N): N; when Lookup.Missing: 0; else 1; end case; end function;",
            "else",
        ),
        (
            "function Read(Value: Lookup of integer): string; begin return case Value of when Lookup.Found(const N): N; when Lookup.Missing: ''; end case; end function;",
            "different types",
        ),
        (
            "function Read(Value: Lookup of integer): integer; begin return case Value of when Lookup.Found(const N): N; end case; end function;",
            "Missing",
        ),
        (
            "type TextLookup = Lookup of string; function Read(Value: Lookup of integer): integer; begin return case Value of when TextLookup.Found(const N): 1; when Lookup.Missing: 0; end case; end function;",
            "mismatch",
        ),
        (
            "type Other of T = enum Found(Value: T); Missing; end enum; function Read(Value: Lookup of integer): integer; begin return case Value of when Other.Found(const N): 1; when Lookup.Missing: 0; end case; end function;",
            "mismatch",
        ),
    ] {
        let errors = check_errors(&format!("program T; {ENUMS} {source} begin end."));
        assert!(
            errors.iter().any(|error| error.message.contains(message)),
            "{errors:?}"
        );
    }
}

#[test]
fn direct_mutual_and_record_enum_recursion_preserve_parameter_positions() {
    check_ok("program T;
type A of (K, V) = enum Stop; Next(Value: B of (K, V)); end enum;
type B of (X, Y) = enum Stop; Next(Value: A of (X, Y)); end enum;
type Entry of T = record Value: T; Next: Link of T; end record;
type Link of U = enum Stop; More(Value: Entry of U); end enum;
begin const X: A of (integer, string) := A.Next(B.Next(A.Stop)); const E: Entry of integer := Entry(Value := 1, Next := Link.Stop); end.");
    for declaration in [
        "type Bad of T = enum Stop; Next(Value: Bad of array of T); end enum;",
        "type Bad of T = enum Stop; Next(Value: Bad of integer); end enum;",
        "type Bad of (K, V) = enum Stop; Next(Value: Bad of (V, K)); end enum;",
        "type A of T = enum Stop; Next(Value: B of T); end enum; type B of U = enum Stop; Next(Value: A of Option of U); end enum;",
    ] {
        let errors = check_errors(&format!("program T; {declaration} begin end."));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("unchanged")),
            "{errors:?}"
        );
    }
    for declaration in [
        "type Bad of T = enum Next(Value: Bad of T); end enum;",
        "type A of T = enum Next(Value: B of T); end enum; type B of U = record Next: A of U; end record;",
    ] {
        let errors = check_errors(&format!("program T; {declaration} begin end."));
        assert!(
            errors.iter().any(|error| error.message.contains("finite")),
            "{errors:?}"
        );
    }
}

#[test]
fn nested_instances_do_not_hide_noncomparable_or_task_payloads() {
    let errors = check_errors(&format!(
        "program T; {ENUMS} begin const X: Lookup of Lookup of array of integer := Lookup.Found(Lookup.Found([1])); const Equal: boolean := X = X; end."
    ));
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:?}"
    );
    let errors = check_errors(&format!(
        "program T; {ENUMS} begin const X: Lookup of Lookup of task of integer := Lookup.Missing; discard X; end."
    ));
    assert!(
        errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD),
        "{errors:?}"
    );
}
