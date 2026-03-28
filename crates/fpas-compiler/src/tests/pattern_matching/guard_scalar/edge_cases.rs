use super::*;

// ===========================================================================
// Guard edge cases — doc: docs/pascal/06-pattern-matching.md § Guard Clauses
// ===========================================================================

#[test]
fn guard_always_true_matches() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if true:
      Std.Console.WriteLn('always')
  else
    Std.Console.WriteLn('never')
  end
end.",
    );
    assert_eq!(out.lines, vec!["always"]);
}

#[test]
fn guard_always_false_falls_through() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if false:
      Std.Console.WriteLn('never');
    X:
      Std.Console.WriteLn('fallthrough')
  end
end.",
    );
    assert_eq!(out.lines, vec!["fallthrough"]);
}

#[test]
fn guard_references_outer_scope_variable() {
    // Guard expression may reference variables from enclosing scope,
    // not only bindings introduced by the label.
    let out = compile_and_run(
        "\
program T;
begin
  var Threshold: integer := 10;
  var X: integer := 15;
  case X of
    X if X > Threshold:
      Std.Console.WriteLn('above');
    X:
      Std.Console.WriteLn('at or below')
  end
end.",
    );
    assert_eq!(out.lines, vec!["above"]);
}

#[test]
fn guard_on_boolean_case() {
    let out = compile_and_run(
        "\
program T;
begin
  var B: boolean := true;
  var Flag: boolean := false;
  case B of
    true if Flag:
      Std.Console.WriteLn('true+flag');
    true:
      Std.Console.WriteLn('true only');
    false:
      Std.Console.WriteLn('false')
  end
end.",
    );
    assert_eq!(out.lines, vec!["true only"]);
}

#[test]
fn guard_on_char_case() {
    let out = compile_and_run(
        "\
program T;
begin
  var C: char := 'A';
  case C of
    'A' if false:
      Std.Console.WriteLn('guarded');
    'A':
      Std.Console.WriteLn('plain A');
    'B':
      Std.Console.WriteLn('B')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["plain A"]);
}

#[test]
fn guard_on_enum_with_outer_scope_variable() {
    // Spec: guard can reference bindings from label + outer scope
    let out = compile_and_run(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
begin
  var MinRadius: real := 5.0;
  var S: Shape := Shape.Circle(3.0);
  case S of
    Shape.Circle(R) if R > MinRadius:
      Std.Console.WriteLn('large');
    Shape.Circle(R):
      Std.Console.WriteLn('small');
    Shape.Rectangle(W, H):
      Std.Console.WriteLn('rect');
    Shape.Point:
      Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["small"]);
}

#[test]
fn guard_on_range_with_else() {
    // Guard on range arm — guard fails → falls to else
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    1..10 if X > 8:
      Std.Console.WriteLn('high range');
    1..10:
      Std.Console.WriteLn('low range')
  else
    Std.Console.WriteLn('out')
  end
end.",
    );
    assert_eq!(out.lines, vec!["low range"]);
}

#[test]
fn guard_on_result_with_outer_scope() {
    let out = compile_and_run(
        "\
program T;
begin
  var Limit: integer := 50;
  var R: Result of integer, string := Ok(42);
  case R of
    Ok(V) if V > Limit:
      Std.Console.WriteLn('above limit');
    Ok(V):
      Std.Console.WriteLn('within limit: ' + Std.Conv.IntToStr(V));
    Error(E):
      Std.Console.WriteLn('error')
  end
end.",
    );
    assert_eq!(out.lines, vec!["within limit: 42"]);
}
