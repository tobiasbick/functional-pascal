use super::*;

#[test]
fn guard_fallthrough_to_unguarded() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    5 if X > 10:
      Std.Console.WriteLn('guarded');
    5:
      Std.Console.WriteLn('unguarded')
  end
end.",
    );
    assert_eq!(out.lines, vec!["unguarded"]);
}

#[test]
fn guard_all_fail_falls_to_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 50;
  case X of
    X if X < 0:
      Std.Console.WriteLn('negative');
    X if X > 100:
      Std.Console.WriteLn('big')
  else
    Std.Console.WriteLn('middle')
  end
end.",
    );
    assert_eq!(out.lines, vec!["middle"]);
}

#[test]
fn guard_multiple_arms_first_matching_wins() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 7;
  case X of
    X if X > 5:
      Std.Console.WriteLn('a');
    X if X > 3:
      Std.Console.WriteLn('b');
    X if X > 0:
      Std.Console.WriteLn('c')
  else
    Std.Console.WriteLn('d')
  end
end.",
    );
    assert_eq!(out.lines, vec!["a"]);
}
