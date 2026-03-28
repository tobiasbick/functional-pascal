use super::*;

#[test]
fn nested_pattern_mixed_flat_and_nested_arms() {
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
  var E: Expr := Expr.Num(42);
  case E of
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('add');
    Expr.Num(N):
      WriteLn('num ' + IntToStr(N));
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["num 42"]);
}

// ---------------------------------------------------------------------------
// Positive: all wildcards (no bindings) in nested position
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_inner_variant_mismatch_falls_to_else() {
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
  var E: Expr := Expr.Add(Expr.Mul(Expr.Num(1), Expr.Num(2)), Expr.Num(3));
  case E of
    Expr.Add(Expr.Add(Expr.Num(A), Expr.Num(B)), Expr.Num(C)):
      WriteLn('nested add');
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('simple add');
  else
    WriteLn('complex')
  end
end.",
    );
    assert_eq!(out.lines, vec!["complex"]);
}

// ---------------------------------------------------------------------------
// Edge: same binding name in different arms (no conflict)
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_same_binding_name_different_arms() {
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
  var E: Expr := Expr.Num(77);
  case E of
    Expr.Add(Expr.Num(N), _):
      WriteLn('add ' + IntToStr(N));
    Expr.Num(N):
      WriteLn('num ' + IntToStr(N));
  end
end.",
    );
    assert_eq!(out.lines, vec!["num 77"]);
}

// ---------------------------------------------------------------------------
// Edge: guard failure with many bindings cleans up correctly
// ---------------------------------------------------------------------------

#[test]
fn nested_pattern_same_root_different_inner() {
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
    Expr.Add(Expr.Num(A), Expr.Num(B)):
      WriteLn('flat add');
    Expr.Add(L, R):
      WriteLn('complex add');
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["complex add"]);
}
