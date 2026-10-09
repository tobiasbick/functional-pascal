//! Reference calls and live referent mutation. See `docs/pascal/tools/debugger.md`.

use super::*;
use fpas_vm::{DebugAssignmentSelector, DebugAssignmentTarget};

fn stopped(source: &str, marker: &str) -> (DebugSession, u64) {
    let line = source
        .lines()
        .position(|line| line.contains(marker))
        .expect("marker")
        + 1;
    let mut session = DebugSession::new(compile(source)).expect("session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: line as u32,
            column: None,
        })
        .expect("breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session.continue_execution().expect("stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    (session, frame)
}

fn target(name: &str) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: name.into(),
        selectors: Vec::new(),
    }
}

fn reference(name: &str) -> DebugExpression {
    DebugExpression::VarArgument(target(name))
}

const SOURCE: &str = "program DebugReferences;
type Box = record Value: integer; end record;
function Add(var Item: integer; Amount: integer): integer;
begin
  Item := Item + Amount;
  return Item;
end function;
function Two(var First: integer; var Second: integer): integer;
begin
  panic('invalid arguments must not enter the body');
  return First + Second;
end function;
begin
  var Item: integer := 4;
  const Fixed: integer := 3;
  var Fraction: real := 2.0;
  var Holder: Box := Box(Value := 5);
  var Items: array of integer := [6, 7];
  var Anchor: integer := 0;
  Anchor := Anchor + 1;
end.";

#[test]
fn explicit_references_mutate_only_the_detached_storage() {
    let (mut session, frame) = stopped(SOURCE, "Anchor := Anchor");
    let before = session.last_stop().clone();
    let result = session
        .evaluate(
            &call("Add", vec![reference("Item"), DebugExpression::Integer(3)]),
            Some(frame),
        )
        .expect("var call");
    assert_eq!(result.value, "7");
    let result = session
        .evaluate(
            &call(
                "Add",
                vec![
                    DebugExpression::NamedArgument {
                        name: "Amount".into(),
                        value: Box::new(DebugExpression::Integer(2)),
                    },
                    DebugExpression::NamedArgument {
                        name: "Item".into(),
                        value: Box::new(reference("Item")),
                    },
                ],
            ),
            Some(frame),
        )
        .expect("named var call");
    assert_eq!(result.value, "6");
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Item".into()), Some(frame))
            .expect("live item")
            .value,
        "4"
    );
    assert_eq!(session.last_stop(), &before);
}

#[test]
fn record_fields_and_array_elements_are_valid_reference_designators() {
    let (mut session, frame) = stopped(SOURCE, "Anchor := Anchor");
    for (root, selector, expected) in [
        (
            "Holder",
            DebugAssignmentSelector::Field("Value".into()),
            "7",
        ),
        (
            "Items",
            DebugAssignmentSelector::Index(DebugExpression::Integer(1)),
            "9",
        ),
    ] {
        let argument = DebugExpression::VarArgument(DebugAssignmentTarget {
            root: root.into(),
            selectors: vec![selector],
        });
        assert_eq!(
            session
                .evaluate(
                    &call("Add", vec![argument, DebugExpression::Integer(2)]),
                    Some(frame)
                )
                .expect("descendant var")
                .value,
            expected
        );
    }
}

#[test]
fn const_types_and_aliases_are_checked_before_execution() {
    let (mut session, frame) = stopped(SOURCE, "Anchor := Anchor");
    let before = session.last_stop().clone();
    for (expression, expected) in [
        (
            call("Add", vec![reference("Fixed"), DebugExpression::Integer(1)]),
            DebugErrorKind::VariableNotMutable,
        ),
        (
            call(
                "Add",
                vec![reference("Fraction"), DebugExpression::Integer(1)],
            ),
            DebugErrorKind::EvaluationType,
        ),
        (
            call("Two", vec![reference("Item"), reference("Item")]),
            DebugErrorKind::EvaluationType,
        ),
        (
            call("Add", vec![reference("Item"), reference("Items")]),
            DebugErrorKind::EvaluationType,
        ),
    ] {
        let failure = session
            .evaluate(&expression, Some(frame))
            .expect_err("invalid var call");
        assert_eq!(failure.kind, expected, "{failure:?}");
        assert!(
            !failure.message.contains("invalid arguments must"),
            "{failure:?}"
        );
        assert_eq!(session.last_stop(), &before);
    }
}

#[test]
fn setting_a_var_parameter_writes_through_and_invalidates_handles_only_on_success() {
    let source = "program DebugWriteThrough;
var Original: integer := 4;
procedure Change(var Item: integer);
begin
  const WriteMarker: integer := 0;
  Item := Item + 1;
end procedure;
begin
  Change(var Original);
end.";
    let (mut session, frame) = stopped(source, "const WriteMarker");
    let before = session.last_stop().clone();
    let failure = session
        .set_expression(
            &target("Item"),
            &DebugExpression::String("wrong".into()),
            Some(frame),
        )
        .expect_err("type check");
    assert_eq!(failure.kind, DebugErrorKind::VariableValueType);
    assert_eq!(session.last_stop(), &before);
    let scope = session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == "Parameters")
        .expect("parameters");
    let result = session
        .set_variable(
            scope.variables_reference,
            "Item",
            &DebugExpression::Integer(9),
        )
        .expect("write through");
    assert_eq!(result.value, "9");
    assert_eq!(
        session
            .evaluate(&DebugExpression::Name("Original".into()), None)
            .expect("caller root")
            .value,
        "9"
    );
    assert!(
        session.scopes(frame).is_err(),
        "old frame must expire after commit"
    );
}

#[test]
fn function_value_assignment_preserves_parameter_modes() {
    let source = "program DebugModes;
function ByValue(Item: integer): integer;
begin return Item; end function;
function ByReference(var Item: integer): integer;
begin return Item; end function;
begin
  var ValueHandler: function(Item: integer): integer := ByValue;
  var ReferenceHandler: function(var Item: integer): integer := ByReference;
  var Anchor: integer := 0;
  Anchor := Anchor + 1;
end.";
    let (mut session, frame) = stopped(source, "Anchor := Anchor");
    let before = session.last_stop().clone();
    for (destination, source) in [
        ("ValueHandler", "ReferenceHandler"),
        ("ReferenceHandler", "ValueHandler"),
    ] {
        let failure = session
            .set_expression(
                &target(destination),
                &DebugExpression::Name(source.into()),
                Some(frame),
            )
            .expect_err("mode mismatch");
        assert_eq!(
            failure.kind,
            DebugErrorKind::VariableValueType,
            "{failure:?}"
        );
        assert_eq!(session.last_stop(), &before);
    }
}

#[test]
fn function_arguments_and_record_fields_preserve_parameter_modes() {
    let source = "program DebugFunctionArguments;
type CallbackBox = record
  Callback: function(var Item: integer): integer;
end record;
function Identity(Item: integer): integer;
begin return Item; end function;
function Increment(var Item: integer): integer;
begin Item := Item + 1; return Item; end function;
function Apply(Handler: function(var Item: integer): integer; var Item: integer): integer;
begin return Item; end function;
begin
  var Item: integer := 4;
  const Reader: function(Item: integer): integer := Identity;
  const Writer: function(var Item: integer): integer := Increment;
  const Box: CallbackBox := CallbackBox(Callback := Writer);
  const Marker: integer := 0;
end.";
    let (mut session, frame) = stopped(source, "const Marker");
    let before = session.last_stop().clone();
    let name = |name: &str| DebugExpression::Name(name.into());
    let constructed = |callback| DebugExpression::Record {
        name: "CallbackBox".into(),
        fields: vec![("Callback".into(), callback)],
    };
    for callee in [
        name("Writer"),
        name("Box.Callback"),
        DebugExpression::Field {
            base: Box::new(constructed(name("Writer"))),
            name: "Callback".into(),
        },
    ] {
        let expression = DebugExpression::Call {
            callee: Box::new(callee),
            arguments: vec![reference("Item")],
        };
        assert_eq!(
            session
                .evaluate(&expression, Some(frame))
                .expect("function var call")
                .value,
            "5"
        );
    }
    assert_eq!(
        session
            .evaluate(
                &call("Apply", vec![name("Writer"), reference("Item")]),
                Some(frame)
            )
            .expect("function argument")
            .value,
        "4"
    );
    for expression in [
        call("Apply", vec![name("Reader"), reference("Item")]),
        constructed(name("Reader")),
    ] {
        let failure = session
            .evaluate(&expression, Some(frame))
            .expect_err("parameter mode mismatch");
        assert_eq!(failure.kind, DebugErrorKind::EvaluationType, "{failure:?}");
        assert!(failure.message.contains("parameter 1"), "{failure:?}");
    }
    assert_eq!(
        session
            .evaluate(&name("Item"), Some(frame))
            .expect("live storage")
            .value,
        "4"
    );
    assert_eq!(session.last_stop(), &before);
}

#[test]
fn reference_paths_use_storage_after_preceding_argument_evaluations() {
    let source = "program DebugReferenceOrder;
function ClearItems(var Items: array of integer): integer;
begin Items := []; return 0; end function;
function IgnoreItem(var Item: integer): integer;
begin return 42; end function;
function Combine(First: integer; Second: integer): integer;
begin return First + Second; end function;
begin
  var Items: array of integer := [1];
  const Marker: integer := 0;
end.";
    let (mut session, frame) = stopped(source, "const Marker");
    let before = session.last_stop().clone();
    let element = DebugExpression::VarArgument(DebugAssignmentTarget {
        root: "Items".into(),
        selectors: vec![DebugAssignmentSelector::Index(DebugExpression::Integer(0))],
    });
    let expression = call(
        "Combine",
        vec![
            call("ClearItems", vec![reference("Items")]),
            call("IgnoreItem", vec![element]),
        ],
    );
    let failure = session
        .evaluate(&expression, Some(frame))
        .expect_err("removed element");
    assert!(failure.message.contains("out of bounds"), "{failure:?}");
    assert_eq!(
        session
            .evaluate(
                &DebugExpression::Index {
                    base: Box::new(DebugExpression::Name("Items".into())),
                    index: Box::new(DebugExpression::Integer(0)),
                },
                Some(frame)
            )
            .expect("live array")
            .value,
        "1"
    );
    assert_eq!(session.last_stop(), &before);
}
