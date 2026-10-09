//! Typed record construction contracts. See `docs/pascal/language/types/records.md`.

use super::named_arguments::{evaluate, server};

#[test]
fn exact_type_aliases_and_defaults_replace_structural_layout_guessing() {
    let mut server = server(
        "program DebugRecords;
type Point = record X: integer := 7; Y: integer := 8; end record;
type Coordinate = Point;
type Other = record X: integer; Y: integer; end record;
type Empty = record end record;
begin end.",
    );
    for (expression, expected) in [
        ("Point(Y := 2).X", "7"),
        ("Point().Y", "8"),
        ("Coordinate(X := 3).Y", "8"),
        ("Other(Y := 2, X := 1).X", "1"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["result"], expected,
            "{expression}: {response}"
        );
    }
    for (expression, expected) in [
        ("Point()", "Point"),
        ("Coordinate()", "Point"),
        ("Other(X := 1, Y := 2)", "Other"),
        ("Empty()", "Empty"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["type_name"], expected,
            "{expression}: {response}"
        );
    }
}

#[test]
fn supplied_fields_precede_defaults_and_defaults_follow_declaration_order() {
    let mut server = server(
        "program DebugDefaultOrder;
var Counter: integer := 0;
function Start(): integer;
begin Counter := 1; return Counter; end function;
function Next(): integer;
begin Counter := Counter + 1; return Counter; end function;
type Ordered = record
  First: integer := Next();
  Second: integer := Next();
  Explicit: integer;
end record;
begin end.",
    );
    for (expression, expected) in [
        ("Ordered(Explicit := Start()).First", "2"),
        ("Ordered(Explicit := Start()).Second", "3"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["result"], expected,
            "{expression}: {response}"
        );
    }
}

#[test]
fn invalid_fields_are_rejected_before_defaults_run() {
    let mut server = server(
        "program DebugRecordErrors;
function Crash(): integer;
begin panic('default must not run'); return 0; end function;
type Needs = record Value: integer := Crash(); Required: integer; end record;
begin end.",
    );
    for (expression, expected) in [
        ("Needs()", "missing required field"),
        ("Needs(Required := 'wrong')", "type"),
        ("Needs(Unknown := 1)", "no stored field"),
        ("Needs(Required := 1, required := 2)", "more than once"),
        ("Needs(1, 2)", "requires named fields"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(response["success"], false, "{response}");
        assert!(
            response.to_string().contains(expected),
            "{expression}: {response}"
        );
        assert!(
            !response.to_string().contains("default must not run"),
            "{response}"
        );
    }
}

#[test]
fn default_routines_follow_the_detached_call_effect_policy() {
    let mut server = server(
        "program DebugDefaultEffects;
uses Std.Console;
function Noisy(): integer;
begin WriteLn('no leak'); return 1; end function;
type NoisyRecord = record Value: integer := Noisy(); end record;
begin end.",
    );
    let response = evaluate(&mut server, "NoisyRecord()");
    assert_eq!(response["success"], false, "{response}");
    assert_eq!(
        response["error"]["code"], "call_effect_forbidden",
        "{response}"
    );
}
