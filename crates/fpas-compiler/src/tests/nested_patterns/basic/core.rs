use super::*;

#[test]
fn nested_pattern_simple() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;

begin
  var V: Outer := Outer.Wrap(Inner.A(42));
  case V of
    Outer.Wrap(Inner.A(X)):
      WriteLn('A ' + IntToStr(X));
    Outer.Wrap(Inner.B):
      WriteLn('B');
    Outer.Empty:
      WriteLn('empty')
  end
end.",
    );
    assert_eq!(out.lines, vec!["A 42"]);
}

#[test]
fn nested_pattern_second_variant() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;

begin
  var V: Outer := Outer.Wrap(Inner.B);
  case V of
    Outer.Wrap(Inner.A(X)):
      WriteLn('A');
    Outer.Wrap(Inner.B):
      WriteLn('B');
    Outer.Empty:
      WriteLn('empty')
  end
end.",
    );
    assert_eq!(out.lines, vec!["B"]);
}

#[test]
fn nested_pattern_recursive_enum() {
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
  var E: Expr := Expr.Add(Expr.Num(3), Expr.Num(4));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn(IntToStr(A) + ' + ' + IntToStr(B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["3 + 4"]);
}

#[test]
fn nested_pattern_fallthrough_to_else() {
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
  var E: Expr := Expr.Add(Expr.Add(Expr.Num(1), Expr.Num(2)), Expr.Num(3));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('simple add');
    Expr.Num(N):
      WriteLn('num');
  else
    WriteLn('complex')
  end
end.",
    );
    assert_eq!(out.lines, vec!["complex"]);
}

#[test]
fn nested_pattern_exhaustive_no_else() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;

begin
  var V: Outer := Outer.Empty;
  case V of
    Outer.Wrap(Inner.A(X)):
      WriteLn('A ' + IntToStr(X));
    Outer.Wrap(Inner.B):
      WriteLn('B');
    Outer.Empty:
      WriteLn('empty')
  end
end.",
    );
    assert_eq!(out.lines, vec!["empty"]);
}
