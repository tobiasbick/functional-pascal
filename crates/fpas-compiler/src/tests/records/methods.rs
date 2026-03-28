use super::*;
#[test]
fn record_method_function_no_extra_args() {
    let out = compile_and_run(
        "\
program RecMethodBasic;
type Point = record
  X: integer;
  Y: integer;
  function Sum(Self: Point): integer;
  begin
    return Self.X + Self.Y
  end;
end;
begin
  var P: Point := record X := 3; Y := 7; end;
  Std.Console.WriteLn(P.Sum())
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}
#[test]
fn record_method_function_with_extra_args() {
    let out = compile_and_run(
        "\
program RecMethodArgs;
type Point = record
  X: integer;
  Y: integer;
  function Add(Self: Point; Other: Point): integer;
  begin
    return Self.X + Other.X + Self.Y + Other.Y
  end;
end;
begin
  var A: Point := record X := 1; Y := 2; end;
  var B: Point := record X := 10; Y := 20; end;
  Std.Console.WriteLn(A.Add(B))
end.",
    );
    assert_eq!(out.lines, vec!["33"]);
}
#[test]
fn record_method_procedure() {
    let out = compile_and_run(
        "\
program RecMethodProc;
type Greeter = record
  Name: string;
  procedure SayHello(Self: Greeter);
  begin
    Std.Console.WriteLn('Hello, ' + Self.Name + '!')
  end;
end;
begin
  var G: Greeter := record Name := 'World'; end;
  G.SayHello()
end.",
    );
    assert_eq!(out.lines, vec!["Hello, World!"]);
}
#[test]
fn record_multiple_methods() {
    let out = compile_and_run(
        "\
program RecMultiMethod;
type Rect = record
  W: integer;
  H: integer;
  function Area(Self: Rect): integer;
  begin
    return Self.W * Self.H
  end;
  function Perimeter(Self: Rect): integer;
  begin
    return 2 * (Self.W + Self.H)
  end;
end;
begin
  var R: Rect := record W := 5; H := 3; end;
  Std.Console.WriteLn(R.Area());
  Std.Console.WriteLn(R.Perimeter())
end.",
    );
    assert_eq!(out.lines, vec!["15", "16"]);
}
#[test]
fn method_body_uses_std_conv() {
    let out = compile_and_run(
        "\
program MethodStdConv;
type Item = record
  Value: integer;
  function AsStr(Self: Item): string;
  begin
    return 'v=' + Std.Conv.IntToStr(Self.Value)
  end;
end;
begin
  var I: Item := record Value := 42; end;
  Std.Console.WriteLn(I.AsStr())
end.",
    );
    assert_eq!(out.lines, vec!["v=42"]);
}
