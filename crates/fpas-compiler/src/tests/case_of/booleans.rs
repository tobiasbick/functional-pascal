use super::super::*;

#[test]
fn case_boolean_true() {
    let out = compile_and_run(
        "\
program T;
begin
  var B: boolean := true;
  case B of
    true: Std.Console.WriteLn('yes');
    false: Std.Console.WriteLn('no')
  end
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}

#[test]
fn case_boolean_false() {
    let out = compile_and_run(
        "\
program T;
begin
  var B: boolean := false;
  case B of
    true: Std.Console.WriteLn('yes');
    false: Std.Console.WriteLn('no')
  end
end.",
    );
    assert_eq!(out.lines, vec!["no"]);
}

#[test]
fn case_boolean_with_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var B: boolean := true;
  case B of
    false: Std.Console.WriteLn('no')
  else
    Std.Console.WriteLn('fallback')
  end
end.",
    );
    assert_eq!(out.lines, vec!["fallback"]);
}
