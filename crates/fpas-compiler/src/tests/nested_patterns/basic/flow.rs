use super::*;

#[test]
fn nested_pattern_in_function() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;

function Eval(E: Expr): integer;
begin
  case E of
    Expr.Num(N):
      return N;
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      return A + B;
  else
    return -1
  end
end;

begin
  WriteLn(IntToStr(Eval(Expr.Add(Expr.Num(3), Expr.Num(4)))))
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn nested_pattern_multiple_case_statements() {
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
  var A: Expr := Expr.Add(Expr.Num(1), Expr.Num(2));
  var B: Expr := Expr.Num(99);
  case A of
    Expr.Add(Expr.Num(X), Expr.Num(Y)):
      WriteLn(IntToStr(X + Y));
  else
    WriteLn('?')
  end;
  case B of
    Expr.Num(N):
      WriteLn(IntToStr(N));
  else
    WriteLn('?')
  end
end.",
    );
    assert_eq!(out.lines, vec!["3", "99"]);
}

#[test]
fn nested_pattern_arm_ordering() {
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
  var E: Expr := Expr.Mul(Expr.Num(3), Expr.Num(5));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('add');
    Expr.Mul(Expr.Num(A), Expr.Num(B)):
      WriteLn('mul ' + IntToStr(A) + '*' + IntToStr(B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["mul 3*5"]);
}
