use super::*;

#[test]
fn nested_pattern_wildcard_first_field() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Add(Expr.Num(1), Expr.Num(99));
  case E of
    Expr.Add(_, Expr.Num(B)):
      WriteLn('right=' + IntToStr(B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["right=99"]);
}

#[test]
fn nested_pattern_wildcard_second_field() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Add(Expr.Num(42), Expr.Num(1));
  case E of
    Expr.Add(Expr.Num(A), _):
      WriteLn('left=' + IntToStr(A));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["left=42"]);
}
