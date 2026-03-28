use super::*;
#[test]
fn enum_data_construct_and_match() {
    let out = compile_and_run(
        "\
program EnumData;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
end;
begin
  var S: Shape := Shape.Circle(5.0);
  case S of
    Shape.Circle(R): WriteLn(R);
    Shape.Rectangle(W, H): WriteLn(W)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}
#[test]
fn enum_data_multiple_fields() {
    let out = compile_and_run(
        "\
program EnumMulti;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
end;
begin
  var S: Shape := Shape.Rectangle(10.0, 20.0);
  case S of
    Shape.Circle(R): WriteLn('circle');
    Shape.Rectangle(W, H): begin
      WriteLn(W);
      WriteLn(H)
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["10", "20"]);
}
#[test]
fn enum_data_mixed_simple_and_data_variants() {
    let out = compile_and_run(
        "\
program EnumMixed;
uses Std.Console;
type Shape = enum
  Nothing;
  Circle(Radius: real);
end;
begin
  var S: Shape := Shape.Nothing;
  case S of
    Shape.Nothing: WriteLn('nothing');
    Shape.Circle(R): WriteLn(R)
  end
end.",
    );
    assert_eq!(out.lines, vec!["nothing"]);
}
#[test]
fn enum_data_pass_to_function() {
    let out = compile_and_run(
        "\
program EnumDataFunc;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
end;
function Describe(S: Shape): string;
begin
  case S of
    Shape.Circle(R): return 'circle';
    Shape.Rectangle(W, H): return 'rectangle'
  end;
  return 'unknown'
end;
begin
  WriteLn(Describe(Shape.Circle(1.0)));
  WriteLn(Describe(Shape.Rectangle(2.0, 3.0)))
end.",
    );
    assert_eq!(out.lines, vec!["circle", "rectangle"]);
}
#[test]
fn enum_data_with_integer_fields() {
    let out = compile_and_run(
        "\
program EnumInt;
uses Std.Console;
type Expr = enum
  Lit(Value: integer);
  Add(Left: integer; Right: integer);
end;
begin
  var E: Expr := Expr.Add(3, 4);
  case E of
    Expr.Lit(V): WriteLn(V);
    Expr.Add(L, R): WriteLn(L + R)
  end
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}
#[test]
fn enum_data_return_variant_from_function() {
    let out = compile_and_run(
        "\
program EnumReturn;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Square(Side: real);
end;
function MakeShape(IsRound: boolean): Shape;
begin
  if IsRound then
    return Shape.Circle(2.5)
  else
    return Shape.Square(4.0)
end;
begin
  var S: Shape := MakeShape(false);
  case S of
    Shape.Circle(R): WriteLn(R);
    Shape.Square(L): WriteLn(L)
  end
end.",
    );
    assert_eq!(out.lines, vec!["4"]);
}
