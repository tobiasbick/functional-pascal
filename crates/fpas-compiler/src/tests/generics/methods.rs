//! Runtime tests for generic record methods.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Generic Methods on Records)

use super::*;

#[test]
fn generic_record_method_infers_return_type_from_callback() {
    let out = compile_and_run(
        "\
program GenericMethodMap;
type Box = record
  Value: integer;
  function Map<R>(Self: Box; F: function(X: integer): R): R;
  begin
    return F(Self.Value)
  end;
end;
function ToText(X: integer): string;
begin
  return 'value=' + Std.Conv.IntToStr(X)
end;
begin
  var B: Box := record Value := 42; end;
  Std.Console.WriteLn(B.Map(ToText))
end.",
    );
    assert_eq!(out.lines, vec!["value=42"]);
}

#[test]
fn generic_record_method_can_return_local_generic_variable() {
    let out = compile_and_run(
        "\
program GenericMethodLocalResult;
type Holder = record
  Value: integer;
  function Wrap<R>(Self: Holder; F: function(X: integer): R): R;
  begin
    var Local: R := F(Self.Value);
    return Local
  end;
end;
function Double(X: integer): integer;
begin
  return X * 2
end;
begin
  var H: Holder := record Value := 21; end;
  Std.Console.WriteLn(H.Wrap(Double))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn generic_record_method_on_generic_record_uses_both_type_levels() {
    let out = compile_and_run(
        "\
program GenericMethodNested;
type Wrapper<T> = record
  Value: T;
  function Transform<R>(Self: Wrapper of T; F: function(X: T): R): R;
  begin
    return F(Self.Value)
  end;
end;
function Describe(X: integer): string;
begin
  return 'n=' + Std.Conv.IntToStr(X)
end;
begin
  var W: Wrapper of integer := record Value := 7; end;
  Std.Console.WriteLn(W.Transform(Describe))
end.",
    );
    assert_eq!(out.lines, vec!["n=7"]);
}

#[test]
fn generic_record_method_constraint_violation_is_compile_error() {
    let err = compile_err(
        "\
program GenericMethodConstraint;
type Box = record
  function AddTwo<T: Numeric>(Self: Box; X: T): T;
  begin
    return X
  end;
end;
begin
  var B: Box := record end;
  var S: string := B.AddTwo('hello')
end.",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Numeric") || msg.contains("constraint"),
        "{msg}"
    );
}
