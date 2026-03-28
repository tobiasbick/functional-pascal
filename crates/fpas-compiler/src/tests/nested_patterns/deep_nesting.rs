use super::*;

#[test]
fn nested_pattern_three_levels_deep() {
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
  var E: Expr := Expr.Add(Expr.Add(Expr.Num(1), Expr.Num(2)), Expr.Num(3));
  case E of
    Expr.Add(Expr.Add(Expr.Num(A), Expr.Num(B)), Expr.Num(C)):
      WriteLn(IntToStr(A) + ' ' + IntToStr(B) + ' ' + IntToStr(C));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["1 2 3"]);
}

#[test]
fn nested_pattern_three_levels_right() {
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
  var E: Expr := Expr.Add(Expr.Num(1), Expr.Add(Expr.Num(2), Expr.Num(3)));
  case E of
    Expr.Add(Expr.Num(A), Expr.Add(Expr.Num(B), Expr.Num(C))):
      WriteLn(IntToStr(A) + ' ' + IntToStr(B) + ' ' + IntToStr(C));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["1 2 3"]);
}

// ---------------------------------------------------------------------------
// Positive: mixed flat + nested arms in same case
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_three_field_variant() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;

type
  Color = enum
    Rgb(R: integer; G: integer; B: integer);
    Named(Name: string);
  end;
  Shape = enum
    Rect(C: Color; W: integer; H: integer);
    Circle(C: Color; Radius: integer);
  end;

begin
  var S: Shape := Shape.Rect(Color.Rgb(255, 0, 128), 10, 20);
  case S of
    Shape.Rect(Color.Rgb(R, G, B), W, H):
      WriteLn(IntToStr(R) + ' ' + IntToStr(G) + ' ' + IntToStr(B) + ' ' + IntToStr(W) + 'x' + IntToStr(H));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["255 0 128 10x20"]);
}

// ---------------------------------------------------------------------------
// Positive: nested arm falls through, next arm matches (arm ordering)
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_four_levels() {
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
  var E: Expr := Expr.Add(Expr.Add(Expr.Add(Expr.Num(1), Expr.Num(2)), Expr.Num(3)), Expr.Num(4));
  case E of
    Expr.Add(Expr.Add(Expr.Add(Expr.Num(A), Expr.Num(B)), Expr.Num(C)), Expr.Num(D)):
      WriteLn(IntToStr(A + B + C + D));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

// ---------------------------------------------------------------------------
// Edge: nested check fails mid-way → else branch
// ---------------------------------------------------------------------------
