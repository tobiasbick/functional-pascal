use super::super::*;

// ===========================================================================
// Exhaustiveness: scalar types do NOT require exhaustive matching.
// Spec: "Scalar types (integer, string, char, boolean): else is recommended
// but not required."
// doc: docs/pascal/06-pattern-matching.md § Exhaustiveness Checking / Rules
// ===========================================================================

#[test]
fn exhaustiveness_not_required_on_boolean() {
    // Boolean with only one arm and no else — must compile
    let out = compile_and_run(
        "\
program T;
begin
  var B: boolean := false;
  case B of
    true: Std.Console.WriteLn('yes')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn exhaustiveness_not_required_on_string() {
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'xyz';
  case S of
    'hello': Std.Console.WriteLn('hi')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn exhaustiveness_not_required_on_char() {
    let out = compile_and_run(
        "\
program T;
begin
  var C: char := 'Z';
  case C of
    'A': Std.Console.WriteLn('first')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}
