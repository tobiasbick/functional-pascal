use super::*;

#[test]
fn ref_alias_shares_field_updates() {
    let out = compile_and_run(
        "\
program RefAlias;
type Point = record X: integer; Y: integer; end;
begin
  mutable var Root: ref Point := new Point with X := 1; Y := 2; end;
  var Alias: ref Point := Root;
  Root.X := 41;
  Std.Console.WriteLn(Alias.X)
end.",
    );
    assert_eq!(out.lines, vec!["41"]);
}

#[test]
fn ref_method_receiver_mutation_updates_alias() {
    let out = compile_and_run(
        "\
program RefMethodMut;
type Counter = record
  Value: integer;
  procedure Inc(mutable Self: Counter);
  begin
    Self.Value := Self.Value + 1
  end;
end;
begin
  mutable var CounterRef: ref Counter := new Counter with Value := 1; end;
  var Alias: ref Counter := CounterRef;
  CounterRef.Inc();
  Std.Console.WriteLn(Alias.Value)
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn ref_self_cycle_through_option_can_be_read() {
    let out = compile_and_run(
        "\
program RefSelfCycle;
type Node = record
  Value: integer;
  Parent: Option of ref Node;
end;
begin
  mutable var Root: ref Node := new Node with Value := 7; Parent := None; end;
  Root.Parent := Some(Root);
  case Root.Parent of
    Some(P): Std.Console.WriteLn(P.Value);
    None: Std.Console.WriteLn(0)
  end
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn ref_can_be_returned_from_function() {
    let out = compile_and_run(
        "\
program RefReturn;
type Counter = record Value: integer; end;

function MakeCounter(Value: integer): ref Counter;
begin
  return new Counter with Value := Value; end
end;

begin
  var CounterRef: ref Counter := MakeCounter(9);
  Std.Console.WriteLn(CounterRef.Value)
end.",
    );
    assert_eq!(out.lines, vec!["9"]);
}

#[test]
fn ref_parent_link_can_be_read_after_assignment() {
    let out = compile_and_run(
        "\
program RefParent;
type Node = record
  Value: integer;
  Parent: Option of ref Node;
end;
begin
  mutable var Root: ref Node := new Node with Value := 1; Parent := None; end;
  mutable var Child: ref Node := new Node with Value := 2; Parent := None; end;
  Child.Parent := Some(Root);
  case Child.Parent of
    Some(P): Std.Console.WriteLn(P.Value);
    None: Std.Console.WriteLn(0)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn new_non_record_type_reports_error() {
    let err = compile_err(
        "\
program RefBadNew;
begin
  var X: ref integer := new integer with end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn immutable_ref_field_set_reports_error() {
    let err = compile_err(
        "\
program RefImmutable;
type Point = record X: integer; end;
begin
  var P: ref Point := new Point with X := 1; end;
  P.X := 2
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT);
}

#[test]
fn new_missing_field_reports_error() {
    let err = compile_err(
        "\
program RefMissingField;
type Point = record X: integer; Y: integer; end;
begin
  var P: ref Point := new Point with X := 1; end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_TYPE_MISMATCH);
}

#[test]
fn ref_unknown_field_reports_error() {
    let err = compile_err(
        "\
program RefBadField;
type Point = record X: integer; end;
begin
  var P: ref Point := new Point with X := 1; end;
  Std.Console.WriteLn(P.Y)
end.",
    );
    assert!(err.message.to_lowercase().contains("field"));
}