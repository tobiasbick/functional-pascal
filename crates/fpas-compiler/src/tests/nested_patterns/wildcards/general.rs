use super::*;

#[test]
fn nested_pattern_with_wildcard() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
    Mul(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Mul(Expr.Num(0), Expr.Add(Expr.Num(99), Expr.Num(1)));
  case E of
    Expr.Mul(Expr.Num(A), _) if A = 0:
      WriteLn('multiply by zero');
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["multiply by zero"]);
}

#[test]
fn nested_pattern_spec_example() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
    Mul(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Add(Expr.Num(10), Expr.Num(20));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('Simple addition: ' + IntToStr(A) + ' + ' + IntToStr(B));
    Expr.Mul(Expr.Num(0), _):
      WriteLn('Multiply by zero');
  else
    WriteLn('Complex expression')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Simple addition: 10 + 20"]);
}

#[test]
fn nested_pattern_mul_by_zero() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
    Mul(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Mul(Expr.Num(0), Expr.Num(5));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('add');
    Expr.Mul(Expr.Num(0), _):
      WriteLn('Multiply by zero');
  else
    WriteLn('Complex expression')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Multiply by zero"]);
}

#[test]
fn nested_pattern_all_wildcards() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Add(Expr.Num(1), Expr.Num(2));
  case E of
    Expr.Add(_, _):
      WriteLn('is add');
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["is add"]);
}
