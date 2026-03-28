use super::*;

#[test]
fn nested_pattern_value_constraint_with_binding() {
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
  var E: Expr := Expr.Add(Expr.Num(0), Expr.Num(7));
  case E of
    Expr.Add(Expr.Num(0), Expr.Num(B)):
      WriteLn('zero plus ' + IntToStr(B));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["zero plus 7"]);
}

#[test]
fn nested_pattern_value_constraint_no_match() {
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
  var E: Expr := Expr.Add(Expr.Num(5), Expr.Num(7));
  case E of
    Expr.Add(Expr.Num(0), Expr.Num(B)):
      WriteLn('zero plus');
  else
    WriteLn('not zero plus')
  end
end.",
    );
    assert_eq!(out.lines, vec!["not zero plus"]);
}

// ---------------------------------------------------------------------------
// Positive: multiple value constraints in one arm
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_multiple_value_constraints() {
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
  var E: Expr := Expr.Add(Expr.Num(0), Expr.Num(0));
  case E of
    Expr.Add(Expr.Num(0), Expr.Num(0)):
      WriteLn('both zero');
  else
    WriteLn('not both zero')
  end
end.",
    );
    assert_eq!(out.lines, vec!["both zero"]);
}

#[test]
fn nested_pattern_multiple_value_constraints_partial_fail() {
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
  var E: Expr := Expr.Add(Expr.Num(0), Expr.Num(1));
  case E of
    Expr.Add(Expr.Num(0), Expr.Num(0)):
      WriteLn('both zero');
  else
    WriteLn('not both zero')
  end
end.",
    );
    assert_eq!(out.lines, vec!["not both zero"]);
}

// ---------------------------------------------------------------------------
// Positive: fieldless inner variant in nested position
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_string_field() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Color = enum
    Rgb(R: integer; G: integer; B: integer);
    Named(Name: string);
  end;
  Shape = enum
    Colored(C: Color);
  end;

begin
  var S: Shape := Shape.Colored(Color.Named('red'));
  case S of
    Shape.Colored(Color.Named(N)):
      WriteLn('color: ' + N);
    Shape.Colored(Color.Rgb(R, G, B)):
      WriteLn('rgb');
  end
end.",
    );
    assert_eq!(out.lines, vec!["color: red"]);
}

// ---------------------------------------------------------------------------
// Positive: exhaustive case without else (all variants covered)
// ---------------------------------------------------------------------------
