use super::*;

// ===========================================================================
// Guard + exhaustiveness combined edge cases
// ===========================================================================

#[test]
fn guarded_plus_unguarded_covers_variant() {
    let out = compile_and_run(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Red;
  case L of
    Light.Red if false: Std.Console.WriteLn('guarded');
    Light.Red: Std.Console.WriteLn('unguarded');
    Light.Yellow: Std.Console.WriteLn('caution');
    Light.Green: Std.Console.WriteLn('go')
  end
end.",
    );
    assert_eq!(out.lines, vec!["unguarded"]);
}

#[test]
fn guard_in_function_with_return() {
    let out = compile_and_run(
        "\
program T;
type Dir = enum North; South; East; West; end;

function Describe(D: Dir): string;
begin
  case D of
    Dir.North if true: return 'north!';
    Dir.North: return 'north';
    Dir.South: return 'south';
    Dir.East: return 'east';
    Dir.West: return 'west'
  end
end;

begin
  Std.Console.WriteLn(Describe(Dir.North));
  Std.Console.WriteLn(Describe(Dir.South))
end.",
    );
    assert_eq!(out.lines, vec!["north!", "south"]);
}

#[test]
fn guard_with_nested_case() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  var Y: integer := 10;
  case X of
    X if X < 10:
      begin
        case Y of
          Y if Y > 5:
            Std.Console.WriteLn('inner guard')
        else
          Std.Console.WriteLn('inner else')
        end
      end
  else
    Std.Console.WriteLn('outer else')
  end
end.",
    );
    assert_eq!(out.lines, vec!["inner guard"]);
}

#[test]
fn guard_on_result_ok_big_value() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Ok(999);
  case R of
    Ok(V) if V > 100:
      Std.Console.WriteLn('big ' + Std.Conv.IntToStr(V));
    Ok(V):
      Std.Console.WriteLn('small');
    Error(E):
      Std.Console.WriteLn('err')
  end
end.",
    );
    assert_eq!(out.lines, vec!["big 999"]);
}

#[test]
fn guard_on_option_some_negative() {
    let out = compile_and_run(
        "\
program T;
begin
  var O: Option of integer := Some(-3);
  case O of
    Some(V) if V > 0:
      Std.Console.WriteLn('positive');
    Some(V):
      Std.Console.WriteLn('non-positive ' + Std.Conv.IntToStr(V));
    None:
      Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["non-positive -3"]);
}

#[test]
fn exhaustiveness_help_text_present() {
    let err = compile_err(
        "\
program T;
type AB = enum A; B; end;
begin
  var X: AB := AB.A;
  case X of
    AB.A: Std.Console.WriteLn('a')
  end
end.",
    );
    assert!(
        err.help.as_deref().is_some_and(|h| !h.is_empty()),
        "exhaustiveness error must include help text"
    );
}
