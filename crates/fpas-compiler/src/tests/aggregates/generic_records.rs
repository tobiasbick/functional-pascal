//! Generic record erasure, methods, nested values, and constructor evaluation order.

use super::*;

#[test]
fn distinct_instantiations_nested_records_and_generic_methods_execute() {
    assert_succeeds(
        r#"
program T;
type T = string;
type Box of T = record
  Value: T;
  function Get(Self: Box of T): T; begin return Self.Value; end function;
  function Map<R>(Self: Box of T; F: function(Value: T): R): R; begin return F(Self.Value); end function;
  static function Create(Value: T): Box of T; begin return Box(Value := Value); end function;
end record;
type AIntBox = Box of integer;
type Optional of T = record Value: Option of T := None; end record;
type List of T = record Values: array of T; end record;
type Mapper of (T, R) = record Input: T; Transform: function(Value: T): R; end record;
function Make<T>(Value: T): Box of T; begin return Box(Value := Value); end function;
function Empty(): Optional of string; begin return Optional(); end function;
function Double(Value: integer): integer; begin return Value * 2; end function;
function TextOf(Value: integer): string; begin return 'value'; end function;
function GenericText<T>(Value: T): string; begin return 'generic'; end function;
function Identity<T>(Value: T): T; begin return Value; end function;
procedure Accept(Value: Optional of integer); begin if Value.Value.IsSome() then panic('expected None'); end if; end procedure;
procedure AcceptGeneric<T>(Empty: Optional of T; Evidence: T); begin if Empty.Value.IsSome() then panic('expected generic None'); end if; end procedure;
begin
  const I: AIntBox := AIntBox.Create(7);
  const S: Box of string := Box(Value := 'seven');
  const N: Box of Box of integer := Box(Value := Make(9));
  var A: Box of array of integer := Box(Value := [1, 2]);
  A.Value[0] := 4;
  const Wrapped: Box of Option of integer := Box(Value := Some(3));
  const Callback: Box of function(Value: integer): integer := Box(Value := Double);
  const GenericCallback: Box of function(Value: integer): integer := Box(Value := Identity);
  const First: string := Mapper(Input := 1, Transform := GenericText).Transform(2);
  const Second: string := Mapper(Transform := GenericText, Input := 1).Transform(2);
  const Chained: integer := Box(Value := Box(Value := 6)).Get().Value;
  const Changed: Box of string := S with Value := 'changed'; end with;
  const EmptyValue: Optional of string := Empty();
  const Present: Optional of integer := Optional(Value := Some(3));
  const Numbers: List of integer := List(Values := [1, 2]);
  Accept(Optional());
  AcceptGeneric(Optional(), 1);
  AcceptGeneric(Evidence := 1, Empty := Optional());
  if (I.Get() <> 7) or (S.Get() <> 'seven') or (N.Value.Get() <> 9) or (Changed.Value <> 'changed') or EmptyValue.Value.IsSome() then panic('generic records'); end if;
  if I <> Make(7) then panic('generic equality'); end if;
  if (A.Value[0] <> 4) or (Wrapped.Value.Unwrap() <> 3) or (Callback.Value(5) <> 10) then panic('generic field shapes'); end if;
  if (I.Map(TextOf) <> 'value') or (Box.Create(4).Get() <> 4) then panic('generic methods'); end if;
  if (I.Map(GenericText) <> 'generic') or (GenericCallback.Value(5) <> 5) then panic('generic callbacks'); end if;
  if (First <> 'generic') or (Second <> 'generic') then panic('record callback inference'); end if;
  if Chained <> 6 then panic('generic method record result'); end if;
  if (Present.Value.Unwrap() <> 3) or (Numbers.Values[1] <> 2) then panic('generic containers'); end if;
end.
"#,
    );
}

#[test]
fn generic_construction_keeps_supplied_then_default_evaluation_order() {
    assert_succeeds(
        r#"
program T;
var Log: string := '';
function Mark(Text: string): integer; begin Log := Log + Text; return 1; end function;
type Pair of T = record
  First: T;
  Second: T;
  Defaulted: integer := Mark('d');
end record;
begin
  const P: Pair of integer := Pair(Second := Mark('s'), First := Mark('f'));
  if (Log <> 'sfd') or (P.First <> 1) or (P.Defaulted <> 1) then panic('field order'); end if;
end.
"#,
    );
}
