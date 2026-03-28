use super::super::*;

// ===========================================================================
// Edge cases for basic case-of — doc: docs/pascal/06-pattern-matching.md
// ===========================================================================

#[test]
fn case_on_arithmetic_expression() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 4;
  case X + 1 of
    5: Std.Console.WriteLn('five');
    6: Std.Console.WriteLn('six')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["five"]);
}

#[test]
fn case_range_single_value() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    5..5: Std.Console.WriteLn('exact');
    6..10: Std.Console.WriteLn('higher')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["exact"]);
}

#[test]
fn case_overlapping_ranges_first_wins() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 7;
  case X of
    1..10: Std.Console.WriteLn('first');
    5..15: Std.Console.WriteLn('second')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["first"]);
}

#[test]
fn case_multiline_comma_separated_labels() {
    let out = compile_and_run(
        "\
program T;
begin
  var Day: string := 'Sunday';
  case Day of
    'Monday':    Std.Console.WriteLn('Start of week');
    'Friday':    Std.Console.WriteLn('Almost weekend');
    'Saturday',
    'Sunday':    Std.Console.WriteLn('Weekend')
  else
    Std.Console.WriteLn('Midweek')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Weekend"]);
}

#[test]
fn case_empty_only_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 99;
  case X of
  else
    Std.Console.WriteLn('only else')
  end
end.",
    );
    assert_eq!(out.lines, vec!["only else"]);
}

#[test]
fn case_integer_large_values() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 1000000;
  case X of
    0: Std.Console.WriteLn('zero');
    1000000: Std.Console.WriteLn('million')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["million"]);
}

#[test]
fn case_negative_range() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := -3;
  case X of
    -10..-1: Std.Console.WriteLn('negative');
    0: Std.Console.WriteLn('zero');
    1..10: Std.Console.WriteLn('positive')
  end
end.",
    );
    assert_eq!(out.lines, vec!["negative"]);
}

#[test]
fn case_block_arm_in_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: string := 'unknown';
  case X of
    'help':
      begin
        Std.Console.WriteLn('Available commands:');
        Std.Console.WriteLn('  help, quit, run')
      end;
    'quit':
      Std.Console.WriteLn('Goodbye')
  else
    begin
      Std.Console.WriteLn('Unknown command');
      Std.Console.WriteLn('Type help')
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["Unknown command", "Type help"]);
}

#[test]
fn case_enum_in_function_returns() {
    // Spec example: DirectionName function
    let out = compile_and_run(
        "\
program T;
type
  Direction = enum
    North;
    South;
    East;
    West;
  end;

function DirectionName(D: Direction): string;
begin
  case D of
    Direction.North: return 'North';
    Direction.South: return 'South';
    Direction.East:  return 'East';
    Direction.West:  return 'West'
  end
end;

begin
  Std.Console.WriteLn(DirectionName(Direction.North));
  Std.Console.WriteLn(DirectionName(Direction.West))
end.",
    );
    assert_eq!(out.lines, vec!["North", "West"]);
}

#[test]
fn case_boolean_only_true_no_else() {
    // Spec: "Scalar types (integer, string, char, boolean): else is
    // recommended but not required."
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
fn case_string_partial_no_else() {
    // Spec: string is a scalar type — exhaustiveness not required
    let out = compile_and_run(
        "\
program T;
begin
  var S: string := 'xyz';
  case S of
    'abc': Std.Console.WriteLn('matched')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}

#[test]
fn case_char_partial_no_else() {
    // Spec: char is a scalar type — exhaustiveness not required
    let out = compile_and_run(
        "\
program T;
begin
  var C: char := 'Z';
  case C of
    'A': Std.Console.WriteLn('matched')
  end;
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["done"]);
}
