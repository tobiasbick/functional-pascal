//! Concrete record constructors share contextual fields, visibility, and lexical lookup.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_MISSING_RECORD_FIELD, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

const TYPES: &str =
    "type Point = record X: integer; Y: integer := 2; end record; type Position = Point;";

#[test]
fn named_fields_aliases_nested_values_and_generic_routines_are_supported() {
    check_ok(&format!(
        "program T; {TYPES}
      type Box = record Value: Point; Items: array of Point; end record;
      function Make<T>(Value: T): Point; begin return Position(x := 1); end function;
      begin
        const P: Point := Position(y := 3, X := 1);
        const B: Box := Box(Items := [Point(X := 4)], Value := Position(X := 5));
        const C: Box := Box(Value := Point( X := 6 ), Items := []);
        const G: Point := Make(1);
      end."
    ));
}

#[test]
fn invalid_constructor_fields_are_diagnosed() {
    for (call, code, message) in [
        ("Point(1, 2)", SEMA_TYPE_MISMATCH, "requires named fields"),
        ("Point(X := 1, Z := 2)", SEMA_UNKNOWN_NAME, "no field `Z`"),
        (
            "Point(X := 1, x := 2)",
            SEMA_DUPLICATE_DECLARATION,
            "more than once",
        ),
        (
            "Point(Y := 3)",
            SEMA_MISSING_RECORD_FIELD,
            "Required field `X`",
        ),
        ("Point(X := 'wrong')", SEMA_TYPE_MISMATCH, "field `X`"),
    ] {
        let errors = check_errors(&format!(
            "program T; {TYPES} begin const P: Point := {call}; end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == code && error.message.contains(message)),
            "{call}: {errors:#?}"
        );
    }
}

#[test]
fn an_empty_constructor_needs_all_fields_to_have_defaults() {
    check_ok(
        "program T; type Empty = record end record; type Defaulted = record Value: integer := 7; end record; begin const E: Empty := Empty(); const D: Defaulted := Defaulted(); end.",
    );
    let errors = check_errors(&format!(
        "program T; {TYPES} begin const P: Point := Position(); end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_MISSING_RECORD_FIELD),
        "{errors:#?}"
    );
}

#[test]
fn methods_are_not_constructor_fields() {
    let errors = check_errors(
        "program T; type Point = record X: integer; function GetX(Self: Point): integer; begin return Self.X; end function; end record; begin const P: Point := Point(X := 1, GetX := 3); end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("no field `GetX`")),
        "{errors:#?}"
    );
}

#[test]
fn lexical_values_do_not_fall_back_to_an_outer_record_type() {
    check_ok(&format!(
        "program T; {TYPES} function Apply(Point: function(Value: integer): integer): integer; begin return Point(3); end function; begin end."
    ));
    let errors = check_errors(&format!(
        "program T; {TYPES} procedure Probe(Point: integer); begin discard Point(X := 1); end procedure; begin end."
    ));
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH && error.message.contains("not callable")), "{errors:#?}");
}

#[test]
fn types_and_routines_cannot_share_a_case_insensitive_scope_name() {
    for declarations in [
        "type Point = record X: integer; end record; function pOINT(): integer; begin return 1; end function;",
        "function pOINT(): integer; begin return 1; end function; type Point = record X: integer; end record;",
    ] {
        let errors = check_errors(&format!("program T; {declarations} begin end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_DUPLICATE_DECLARATION),
            "{errors:#?}"
        );
    }
}

#[test]
fn constructors_cannot_be_spawned_or_pass_var_fields() {
    let errors = check_errors(&format!(
        "program T; {TYPES} begin const Work: task := go Point(X := 1); end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("constructs a value")),
        "{errors:#?}"
    );
    let errors = check_errors(&format!(
        "program T; {TYPES} begin var N: integer := 1; const P: Point := Point(X := var N); end."
    ));
    assert!(
        errors.iter().any(|error| error.message.contains("var")),
        "{errors:#?}"
    );
}

#[test]
fn record_calls_and_record_returning_routines_have_distinct_metadata() {
    let (program, errors) = fpas_parser::parse(&format!(
        "program T; {TYPES} function Make(): Point; begin return Point(X := 1); end function; begin const P: Point := Make(); end."
    ));
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = crate::analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    assert_eq!(metadata.record_constructions.len(), 1);
}

#[test]
fn computed_constructor_defaults_participate_in_const_classification() {
    check_ok(
        "program T; type Point = record X: integer := 2; end record; begin const P: Point := Point(); case 2 of when P.X: null; end case; end.",
    );
    let errors = check_errors(
        "program T; function GetValue(): integer; begin return 2; end function; type Point = record X: integer := GetValue(); end record; begin const P: Point := Point(); case 2 of when P.X: null; end case; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION),
        "{errors:#?}"
    );
}

#[test]
fn constant_defaults_are_classified_in_their_declaration_environment() {
    check_ok(
        "program T;
      const Seed: integer := 2;
      type Point = record X: integer := Seed; end record;
      procedure Probe(Seed: integer);
      begin const P: Point := Point(); case 2 of when P.X: null; end case; end procedure;
      begin Probe(99); end.",
    );
    let errors = check_errors("program T;
      var Seed: integer := 2;
      type Point = record X: integer := Seed; end record;
      begin const Seed: integer := 1; const P: Point := Point(); case 2 of when P.X: null; end case; end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION),
        "{errors:#?}"
    );
}

#[test]
fn typed_construction_retains_task_bound_and_discard_capture_guarantees() {
    check_ok(
        "program T; type Holder = record Callback: function(): integer := function(): integer begin return 1; end function; end record; begin discard Holder(); const Offset: integer := 2; discard Holder(Callback := function(): integer begin return Offset; end function); end.",
    );
    let errors = check_errors(
        "program T;
      type Holder = record Callback: procedure(); end record;
      begin var N: integer := 0;
        const H: Holder := Holder(Callback := procedure() begin N := N + 1; end procedure);
        go H.Callback();
      end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
    let errors = check_errors("program T; uses Std.Tasks;
      function Work(): integer; begin return 1; end function;
      function Make(Job: task of integer): function(): integer; begin return function(): integer begin return Wait(Job); end function; end function;
      const Callback: function(): integer := Make(go Work());
      type Holder = record Value: function(): integer := Callback; end record;
      begin discard Holder(); end.");
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_UNSAFE_DISCARD),
        "{errors:#?}"
    );
}

#[test]
fn unused_constructor_results_require_consumption_with_valid_discard_hints() {
    let errors = check_errors("program T;
      type Holder = record Callback: function(): integer := function(): integer begin return 1; end function; end record;
      begin Holder(); end.");
    let error = errors
        .iter()
        .find(|error| error.code == fpas_diagnostics::codes::SEMA_UNUSED_FUNCTION_RESULT)
        .expect("unused constructor");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`discard Call();`")),
        "{errors:#?}"
    );
}
