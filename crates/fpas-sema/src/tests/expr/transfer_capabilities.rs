//! Tri-state capture metadata and routine-body isolation.
//! See `docs/pascal/language/functions/closures.md`.

use super::check_ok;
use crate::{TransferCapability, analyze_with_types};

#[test]
fn callable_parameters_are_unknown_but_scalar_captures_are_transferable() {
    let (program, errors) = fpas_parser::parse(
        r#"
program T;
procedure Inspect(Work: procedure(); Value: integer);
begin
  const Invoke: procedure() := procedure() begin Work(); end procedure;
  const Read: function(): integer := function(): integer begin return Value; end function;
end procedure;
begin end.
"#,
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    for (name, expected) in [
        ("Work", TransferCapability::Unknown),
        ("Value", TransferCapability::Transferable),
    ] {
        let info = metadata
            .closure_infos
            .values()
            .find(|info| {
                info.captures
                    .iter()
                    .any(|capture| capture.name.eq_ignore_ascii_case(name))
            })
            .expect("capture");
        assert_eq!(info.capability, expected);
        assert_eq!(info.captures[0].capability, expected);
    }
}

#[test]
fn deferred_capture_completion_replaces_provisional_unknown_with_direct_facts() {
    let (program, errors) = fpas_parser::parse(
        r#"
program T;
procedure Free(); begin end procedure;
procedure Outer();
  procedure First();
  begin
    const Copy: procedure() := Second;
    Work();
  end procedure;
  procedure Second(); begin First(); end procedure;
begin
  const Work: procedure() := Free;
end procedure;
begin end.
"#,
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert_eq!(metadata.nested_routine_captures.len(), 2);
    for info in metadata.nested_routine_captures.values() {
        assert_eq!(
            info.capability,
            TransferCapability::Transferable,
            "{info:#?}"
        );
    }
}

#[test]
fn recursive_completion_keeps_genuine_unknown_parameter_captures() {
    let (program, errors) = fpas_parser::parse(
        r#"
program T;
procedure Outer(Input: procedure());
  procedure First(); begin Second(); Work(); end procedure;
  procedure Second(); begin First(); end procedure;
begin
  const Work: procedure() := Input;
end procedure;
begin end.
"#,
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert_eq!(metadata.nested_routine_captures.len(), 2);
    for info in metadata.nested_routine_captures.values() {
        assert_eq!(info.capability, TransferCapability::Unknown, "{info:#?}");
    }
}

#[test]
fn uncalled_closure_and_named_routine_do_not_change_outer_flow_state() {
    check_ok(
        r#"
program T;
procedure Free(); begin end procedure;
procedure Named();
  procedure Change(); begin Work := Local; end procedure;
begin
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := Free;
  scope discard go Work(); end scope;
end procedure;
begin scope
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := Free;
  const Change: procedure() := procedure() begin Work := Local; end procedure;
  discard go Work();
end scope; end.
"#,
    );
}

#[test]
fn scalar_record_projection_does_not_inherit_unrelated_callable_locality() {
    check_ok(
        r#"
program T;
type Counter = record
  Count: integer;
  function Read(Self: Counter): integer; begin return Self.Count; end function;
end record;
type Bundle = record
  Value: Counter;
  Work: procedure();
end record;
begin scope
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  const Both: Bundle := Bundle(Value := Counter(Count := 1), Work := Local);
  const Read: function(): integer := Both.Value.Read;
  discard go Read();
end scope; end.
"#,
    );
}

#[test]
fn deferred_native_results_follow_retained_inputs() {
    let source = |expression: &str| {
        format!(
            r#"
program T;
uses Std.Tasks as Tasks;
procedure Free(); begin end procedure;
procedure Outer();
  procedure First();
  begin scope
    const Work: procedure() := procedure() begin First(); end procedure;
    const Clean: array of procedure() := {expression};
    const Queue: channel of array of procedure() := Tasks.CreateChannel(1);
    discard Tasks.Send(Queue, Clean);
    Count := Count + 1;
  end scope; end procedure;
begin var Count: integer := 0; end procedure;
begin end.
"#
        )
    };
    check_ok(&source(
        "[Work].Map(function(Item: procedure()): procedure() begin return Free; end function)",
    ));
    let errors = super::check_errors(&source("[Free].Push(Work)"));
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
}

#[test]
fn deferred_scalar_receivers_do_not_keep_unrelated_pending_captures() {
    check_ok(
        r#"
program T;
type Counter = record
  Count: integer;
  function Read(Self: Counter): integer; begin return Self.Count; end function;
end record;
type Bundle = record Value: Counter; Work: procedure(); end record;
procedure Outer();
  procedure First();
  begin scope
    const Work: procedure() := procedure() begin First(); end procedure;
    const Both: Bundle := Bundle(Value := Counter(Count := 1), Work := Work);
    discard go Both.Value.Read();
    Count := Count + 1;
  end scope; end procedure;
begin var Count: integer := 0; end procedure;
begin end.
"#,
    );
}

#[test]
fn deferred_scalar_pattern_capture_keeps_its_own_type_proof() {
    check_ok(
        r#"
program T;
procedure Outer();
  procedure First();
  begin scope
    const Work: procedure() := procedure() begin First(); end procedure;
    const Mixed: Result of (integer, procedure()) := Error(Work);
    case Mixed of
      when Ok(const Number):
        const Read: function(): integer := function(): integer begin return Number; end function;
        discard go Read();
      when Error(_): null;
    end case;
    Count := Count + 1;
  end scope; end procedure;
begin var Count: integer := 0; end procedure;
begin end.
"#,
    );
}

#[test]
fn invoking_local_aliases_and_callbacks_invalidates_captured_callable_storage() {
    for invoke in [
        "Change();",
        "const Alias := Change; Alias();",
        "[1].ForEach(procedure(Item: integer) begin Work := Local; end procedure);",
    ] {
        let source = format!(
            r#"
program T;
procedure Free(); begin end procedure;
begin scope
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := Free;
  const Change: procedure() := procedure() begin Work := Local; end procedure;
  {invoke}
  const Snapshot: procedure() := Work;
  const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
end scope; end.
"#
        );
        assert_capture_capability(&source, "Snapshot", TransferCapability::Unknown);
    }
}

#[test]
fn invoked_named_helper_invalidates_captured_callable_storage() {
    assert_capture_capability(
        r#"
program T;
procedure Free(); begin end procedure;
procedure Outer();
  procedure Change(); begin Work := Local; end procedure;
begin
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := Free;
  Change();
  const Snapshot: procedure() := Work;
  const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
end procedure;
begin end.
"#,
        "Snapshot",
        TransferCapability::Unknown,
    );
}

#[test]
fn routine_results_do_not_freeze_outer_mutable_contents() {
    for (initial, next) in [("Free", "Local"), ("Local", "Free")] {
        let source = format!(
            r#"
program T;
procedure Free(); begin end procedure;
begin scope
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := {initial};
  const Get: function(): procedure() := function(): procedure() begin return Work; end function;
  Work := {next};
  const Snapshot: procedure() := Get();
  const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
  discard go Snapshot();
end scope; end.
"#
        );
        assert_capture_capability(&source, "Snapshot", TransferCapability::Unknown);
    }
}

fn assert_capture_capability(source: &str, capture: &str, expected: TransferCapability) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    let info = metadata
        .closure_infos
        .values()
        .find(|info| {
            info.captures
                .iter()
                .any(|value| value.name.eq_ignore_ascii_case(capture))
        })
        .expect("capturing closure");
    assert_eq!(info.capability, expected, "{info:#?}");
}

#[test]
fn ordinary_source_calls_invalidate_mutable_callable_globals() {
    assert_capture_capability(
        r#"
program T;
procedure Free(); begin end procedure;
var Work: procedure() := Free;
function MakeLocal(): procedure();
begin
  var Count: integer := 0;
  return procedure() begin Count := Count + 1; end procedure;
end function;
procedure Change(); begin Work := MakeLocal(); end procedure;
begin scope
  Change();
  const Snapshot: procedure() := Work;
  const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
end scope; end.
"#,
        "Snapshot",
        TransferCapability::Unknown,
    );
}

#[test]
fn transferable_source_wrappers_can_mutate_local_captures_through_stored_callbacks() {
    assert_capture_capability(
        r#"
program T;
procedure Free(); begin end procedure;
var Saved: procedure() := Free;
procedure InvokeSaved(); begin Saved(); end procedure;
begin scope
  var Count: integer := 0;
  const Local: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Work: procedure() := Free;
  const Change: procedure() := procedure() begin Work := Local; end procedure;
  Saved := Change;
  Work := Free;
  InvokeSaved();
  const Snapshot: procedure() := Work;
  const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
end scope; end.
"#,
        "Snapshot",
        TransferCapability::Unknown,
    );
}

#[test]
fn mutable_assignment_of_pending_named_callable_has_no_transferable_proof() {
    for (overwrite, expected) in [
        ("", TransferCapability::Unknown),
        ("Work := Free;", TransferCapability::Transferable),
    ] {
        let source = format!(
            r#"
program T;
procedure Free(); begin end procedure;
procedure Outer();
  procedure Early();
  begin scope
    var Work: procedure() := Free;
    Work := Later;
    {overwrite}
    const Snapshot: procedure() := Work;
    const Inspect: procedure() := procedure() begin Snapshot(); end procedure;
    discard go Work();
  end scope; end procedure;
  procedure Later(); begin Count := Count + 1; end procedure;
begin var Count: integer := 0; end procedure;
begin end.
"#
        );
        assert_capture_capability(&source, "Snapshot", expected);
    }
}

#[test]
fn whole_binding_overwrite_drops_stale_pending_initializer_targets() {
    check_ok(
        r#"
program T;
procedure Free(); begin end procedure;
procedure Outer();
  procedure Early();
  begin scope
    var Work: procedure() := Later;
    Work := Free;
    discard go Work();
  end scope; end procedure;
  procedure Later(); begin Count := Count + 1; end procedure;
begin var Count: integer := 0; end procedure;
begin end.
"#,
    );
}
