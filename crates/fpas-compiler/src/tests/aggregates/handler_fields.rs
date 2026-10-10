//! Optional record handlers use ordinary calls and record value semantics.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::assert_succeeds;

#[test]
fn record_methods_and_optional_handlers_execute() {
    assert_succeeds(
        r#"
program RegisterHandlers;
var LastValue: integer := 0;
type Counter = record
  Value: integer;
  function Double(Self: Counter): integer;
  begin
    return Self.Value * 2;
  end function;
end record;
type Button = record
  OnValue: Option of procedure(Value: integer) := None;
  procedure Notify(Self: Button; Value: integer);
  begin
    if Self.OnValue is Some(const Handler) then
      Handler(Value);
    end if;
  end procedure;
end record;
procedure Remember(Value: integer);
begin
  LastValue := Value;
end procedure;
begin
  const C: Counter := Counter(Value := 6);
  if C.Double() <> 12 then panic('method mismatch'); end if;
  var B: Button := Button();
  B.Notify(99);
  if B.OnValue.IsSome() or LastValue <> 0 then panic('unexpected handler'); end if;
  B.OnValue := Some(Remember);
  if not B.OnValue.IsSome() then panic('missing handler'); end if;
  B.Notify(17);
  if LastValue <> 17 then panic('handler call mismatch'); end if;
  B.OnValue := (None);
  B.Notify(99);
  if B.OnValue.IsSome() or LastValue <> 17 then panic('handler was not cleared'); end if;
end.
"#,
    );
}

#[test]
fn handler_fields_retain_bound_methods_in_record_copies() {
    assert_succeeds(
        r#"
program RegisterBoundHandler;
type Counter = record
  Base: integer;
  function Add(Self: Counter; Value: integer): integer;
  begin
    return Self.Base + Value;
  end function;
end record;
type Source = record
  OnValue: Option of function(Value: integer): integer := None;
end record;
begin
  var C: Counter := Counter(Base := 12);
  var S: Source := Source();
  const Empty: Source := S;
  S.OnValue := Some(C.Add);
  const Installed: Source := S;
  const Call: function(Value: integer): integer := S.OnValue.Unwrap();
  C.Base := 99;
  if Call(8) <> 20 then panic('bound handler mismatch'); end if;
  S.OnValue := None;
  if S.OnValue.IsSome() or Empty.OnValue.IsSome() then panic('record copy changed'); end if;
  const Retained: function(Value: integer): integer := Installed.OnValue.Unwrap();
  if Retained(8) <> 20 then panic('copied handler changed'); end if;
end.
"#,
    );
}
