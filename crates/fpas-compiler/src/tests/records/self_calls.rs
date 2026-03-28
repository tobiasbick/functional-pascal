use super::*;
#[test]
fn method_calls_own_method_on_self() {
    let out = compile_and_run(
        "\
program SelfMethodCall;
type Rect = record
  W: integer;
  H: integer;
  function Area(Self: Rect): integer;
  begin
    return Self.W * Self.H
  end;
  function DoubleArea(Self: Rect): integer;
  begin
    return Self.Area() * 2
  end;
end;
begin
  var R: Rect := record W := 5; H := 3; end;
  Std.Console.WriteLn(R.DoubleArea())
end.",
    );
    assert_eq!(out.lines, vec!["30"]);
}
#[test]
fn method_calls_other_method_on_self_with_args() {
    let out = compile_and_run(
        "\
program SelfMethodWithArgs;
type Vec2 = record
  X: integer;
  Y: integer;
  function Add(Self: Vec2; Other: Vec2): integer;
  begin
    return Self.X + Other.X + Self.Y + Other.Y
  end;
  function AddSelf(Self: Vec2): integer;
  begin
    return Self.Add(Self)
  end;
end;
begin
  var V: Vec2 := record X := 4; Y := 6; end;
  Std.Console.WriteLn(V.AddSelf())
end.",
    );
    assert_eq!(out.lines, vec!["20"]);
}
#[test]
fn procedure_method_calls_function_method_on_self() {
    let out = compile_and_run(
        "\
program ProcCallsFuncSelf;
type Counter = record
  Value: integer;
  function AsStr(Self: Counter): string;
  begin
    return Std.Conv.IntToStr(Self.Value)
  end;
  procedure Print(Self: Counter);
  begin
    Std.Console.WriteLn('count=' + Self.AsStr())
  end;
end;
begin
  var C: Counter := record Value := 99; end;
  C.Print()
end.",
    );
    assert_eq!(out.lines, vec!["count=99"]);
}
#[test]
fn method_chain_via_intermediate_variable() {
    let out = compile_and_run(
        "\
program MethodReturnRecord;
type Num = record
  V: integer;
  function Inc(Self: Num): Num;
  begin
    var NewV: integer := Self.V + 1;
    return record V := NewV; end
  end;
  function Show(Self: Num): string;
  begin
    return Std.Conv.IntToStr(Self.V)
  end;
end;
begin
  var A: Num := record V := 10; end;
  var B: Num := A.Inc();
  var C: Num := B.Inc();
  Std.Console.WriteLn(C.Show())
end.",
    );
    assert_eq!(out.lines, vec!["12"]);
}
#[test]
fn method_on_self_boolean_result() {
    let out = compile_and_run(
        "\
program SelfBoolMethod;
type Box = record
  W: integer;
  H: integer;
  function IsSquare(Self: Box): boolean;
  begin
    return Self.W = Self.H
  end;
  function Describe(Self: Box): string;
  begin
    if Self.IsSquare() then
      return 'square'
    else
      return 'rect'
  end;
end;
begin
  var S: Box := record W := 4; H := 4; end;
  var R: Box := record W := 3; H := 5; end;
  Std.Console.WriteLn(S.Describe());
  Std.Console.WriteLn(R.Describe())
end.",
    );
    assert_eq!(out.lines, vec!["square", "rect"]);
}
