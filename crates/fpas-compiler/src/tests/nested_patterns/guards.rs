use super::*;

#[test]
fn nested_pattern_with_guard() {
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
  var E: Expr := Expr.Add(Expr.Num(100), Expr.Num(200));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)) if A + B > 500:
      WriteLn('large sum');
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('sum: ' + IntToStr(A + B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["sum: 300"]);
}

#[test]
fn nested_pattern_guard_triggers() {
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
  var E: Expr := Expr.Add(Expr.Num(400), Expr.Num(200));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)) if A + B > 500:
      WriteLn('large sum: ' + IntToStr(A + B));
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('small sum');
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["large sum: 600"]);
}

// ---------------------------------------------------------------------------
// Positive: 3 levels of nesting depth
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_guard_both_sides() {
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
  var E: Expr := Expr.Add(Expr.Num(10), Expr.Num(20));
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)) if A = B:
      WriteLn('equal');
    Expr.Add(Expr.Num(A), Expr.Num(B)) if A < B:
      WriteLn('left smaller: ' + IntToStr(A) + '<' + IntToStr(B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["left smaller: 10<20"]);
}

// ---------------------------------------------------------------------------
// Positive: string value in nested enum field
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_guard_failure_cleanup_many_bindings() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Color = enum
    Rgb(R: integer; G: integer; B: integer);
  end;
  Shape = enum
    Rect(C: Color; W: integer; H: integer);
  end;

begin
  var S: Shape := Shape.Rect(Color.Rgb(10, 20, 30), 100, 200);
  case S of
    Shape.Rect(Color.Rgb(R, G, B), W, H) if R > 100:
      WriteLn('bright');
    Shape.Rect(Color.Rgb(R, G, B), W, H):
      WriteLn(IntToStr(R) + ' ' + IntToStr(W) + 'x' + IntToStr(H));
  end
end.",
    );
    assert_eq!(out.lines, vec!["10 100x200"]);
}

// ---------------------------------------------------------------------------
// Edge: nested pattern where root variant matches but nested doesn't, then
// a later arm with same root variant matches
// ---------------------------------------------------------------------------
