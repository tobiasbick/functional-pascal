use super::*;

#[test]
fn nested_pattern_non_exhaustive_error() {
    let e = compile_err(
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
  var V: Outer := Outer.Empty;
  case V of
    Outer.Wrap(Inner.A(X)):
      WriteLn('a')
  end
end.",
    );
    assert!(
        e.message.contains("Non-exhaustive") || e.message.contains("missing"),
        "expected non-exhaustive error, got: {}",
        e.message
    );
}

// ---------------------------------------------------------------------------
// Negative: guard must be boolean
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_guard_non_boolean_error() {
    let e = compile_err(
        "\
program T;

type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
  end;

begin
  var E: Expr := Expr.Num(1);
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)) if A + B:
      E := Expr.Num(0)
  else
    E := Expr.Num(0)
  end
end.",
    );
    assert!(
        e.message.contains("boolean") || e.message.contains("Boolean"),
        "expected boolean guard error, got: {}",
        e.message
    );
}

// ---------------------------------------------------------------------------
// Edge: deeply nested (4 levels) pattern
// ---------------------------------------------------------------------------
