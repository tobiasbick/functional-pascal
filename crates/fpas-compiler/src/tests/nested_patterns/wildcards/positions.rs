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

// ---------------------------------------------------------------------------
// Spec § Wildcards: wildcard first + value constraint second
// doc: docs/pascal/06-pattern-matching.md
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_wildcard_first_value_constraint_second() {
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
  var E: Expr := Expr.Mul(Expr.Num(99), Expr.Num(0));
  case E of
    Expr.Mul(_, Expr.Num(0)):
      WriteLn('Multiply by zero');
  else
    WriteLn('Other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Multiply by zero"]);
}

#[test]
fn nested_pattern_wildcard_first_value_constraint_second_no_match() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Expr = enum
    Num(Value: integer);
    Mul(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Mul(Expr.Num(3), Expr.Num(5));
  case E of
    Expr.Mul(_, Expr.Num(0)):
      WriteLn('Multiply by zero');
  else
    WriteLn('Other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Other"]);
}

#[test]
fn nested_pattern_spec_wildcards_both_zero_patterns() {
    // Full spec example from § Wildcards (lines 163–172)
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
  var E1: Expr := Expr.Mul(Expr.Num(0), Expr.Num(42));
  case E1 of
    Expr.Mul(Expr.Num(0), _):
      WriteLn('left zero');
    Expr.Mul(_, Expr.Num(0)):
      WriteLn('right zero');
  else
    WriteLn('Other')
  end;

  var E2: Expr := Expr.Mul(Expr.Num(7), Expr.Num(0));
  case E2 of
    Expr.Mul(Expr.Num(0), _):
      WriteLn('left zero');
    Expr.Mul(_, Expr.Num(0)):
      WriteLn('right zero');
  else
    WriteLn('Other')
  end;

  var E3: Expr := Expr.Mul(Expr.Num(3), Expr.Num(5));
  case E3 of
    Expr.Mul(Expr.Num(0), _):
      WriteLn('left zero');
    Expr.Mul(_, Expr.Num(0)):
      WriteLn('right zero');
  else
    WriteLn('Other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["left zero", "right zero", "Other"]);
}
