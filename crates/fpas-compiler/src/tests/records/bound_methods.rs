//! Bound record method integration tests.
//!
//! **Documentation:** [docs/pascal/language/types/record-methods.md](docs/pascal/language/types/record-methods.md)

use super::*;

#[test]
fn bound_function_method_captures_receiver_by_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
begin
  mutable var C: Counter := record Base := 10; end;
  var AddTen: function(Value: integer): integer := C.Add;
  C := record Base := 100; end;
  WriteLn(IntToStr(AddTen(5)))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn bound_procedure_method_runs() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Box = record
    Label: string;
    procedure Show(Self: Box);
    begin
      WriteLn(Self.Label)
    end;
  end;
begin
  var B: Box := record Label := 'ok'; end;
  var Show: procedure() := B.Show;
  Show()
end.",
    );
    assert_eq!(out.lines, vec!["ok"]);
}

#[test]
fn bound_method_from_postfix_chain() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
    static function Make(Base: integer): Counter;
    begin
      return record Base := Base; end
    end;
  end;
begin
  var AddTen: function(Value: integer): integer := Counter.Make(10).Add;
  WriteLn(IntToStr(AddTen(5)))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn bound_method_passed_as_argument() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
function Apply(F: function(Value: integer): integer; N: integer): integer;
begin
  return F(N)
end;
begin
  var C: Counter := record Base := 7; end;
  WriteLn(IntToStr(Apply(C.Add, 3)))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn bound_method_from_nested_and_indexed_receivers() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
  Holder = record
    Item: Counter;
  end;
begin
  var H: Holder := record Item := record Base := 10; end; end;
  var Items: array of Counter := [record Base := 20; end];
  var AddTen: function(Value: integer): integer := H.Item.Add;
  var AddTwenty: function(Value: integer): integer := Items[0].Add;
  WriteLn(IntToStr(AddTen(1)));
  WriteLn(IntToStr(AddTwenty(2)))
end.",
    );
    assert_eq!(out.lines, vec!["11", "22"]);
}

#[test]
fn generic_bound_method_infers_from_callable_type() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Wrapper = record
    function Identity<T>(Self: Wrapper; Value: T): T;
    begin
      return Value
    end;
  end;
begin
  var W: Wrapper := record end;
  var IdentityInt: function(Value: integer): integer := W.Identity;
  WriteLn(IntToStr(IdentityInt(7)))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn returned_bound_method_outlives_creator_scope() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
function MakeAdder(Base: integer): function(Value: integer): integer;
begin
  var C: Counter := record Base := Base; end;
  return C.Add
end;
begin
  var AddNine: function(Value: integer): integer := MakeAdder(9);
  WriteLn(IntToStr(AddNine(3)))
end.",
    );
    assert_eq!(out.lines, vec!["12"]);
}

#[test]
fn indexed_receiver_is_evaluated_once_when_binding() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
mutable var Calls: integer := 0;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
function NextIndex(): integer;
begin
  Calls := Calls + 1;
  return 0
end;
begin
  var Items: array of Counter := [record Base := 5; end];
  var AddFive: function(Value: integer): integer := Items[NextIndex()].Add;
  WriteLn(IntToStr(AddFive(1)));
  WriteLn(IntToStr(AddFive(2)));
  WriteLn(IntToStr(Calls))
end.",
    );
    assert_eq!(out.lines, vec!["6", "7", "1"]);
}

#[test]
fn field_and_method_cannot_share_a_name() {
    let err = compile_err(
        "program T;
function Double(Value: integer): integer;
begin
  return Value * 2
end;
type
  Handler = record
    Apply: function(Value: integer): integer;
    function Apply(Self: Handler; Value: integer): integer;
    begin
      return Value + 100
    end;
  end;
begin
end.",
    );
    assert!(
        err.message.contains("Duplicate record member"),
        "{}",
        err.message
    );
}

#[test]
fn record_alias_exposes_bound_instance_method() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Conv;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
  CounterAlias = Counter;
begin
  var C: CounterAlias := record Base := 6; end;
  var AddSix: function(Value: integer): integer := C.Add;
  WriteLn(IntToStr(AddSix(3)))
end.",
    );
    assert_eq!(out.lines, vec!["9"]);
}

#[test]
fn rejects_mutable_self_binding_at_compile_time() {
    let err = compile_err(
        "program T;
type
  Counter = record
    Base: integer;
    procedure Inc(mutable Self: Counter);
    begin
      Self.Base := Self.Base + 1
    end;
  end;
begin
  var C: Counter := record Base := 0; end;
  var Op: procedure() := C.Inc
end.",
    );
    assert!(
        err.message.contains("mutable") || err.message.contains("bind"),
        "{}",
        err.message
    );
}
