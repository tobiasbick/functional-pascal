use super::*;

#[test]
fn break_in_else_continue_in_if() {
    let out = compile_and_run(
        "\
program BrkElseContIf;
begin
  for I: integer := 1 to 20 do
  begin
    if I mod 3 = 0 then
      continue
    else if I > 10 then
      break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "4", "5", "7", "8", "10"]);
}

#[test]
fn break_in_if_continue_in_else() {
    let out = compile_and_run(
        "\
program BrkIfContElse;
begin
  mutable var I: integer := 0;
  while true do
  begin
    I := I + 1;
    if I > 5 then
      break
    else if I mod 2 = 0 then
      continue;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}

#[test]
fn multiple_breaks_different_branches() {
    let out = compile_and_run(
        "\
program MultiBrk;
begin
  for I: integer := 1 to 100 do
  begin
    if I = 3 then
    begin
      Std.Console.WriteLn('hit-3');
      break
    end;
    if I = 7 then
    begin
      Std.Console.WriteLn('hit-7');
      break
    end;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "hit-3"]);
}

#[test]
fn break_in_deep_if() {
    let out = compile_and_run(
        "\
program BrkDeepIf;
begin
  for I: integer := 1 to 100 do
  begin
    if I > 2 then
    begin
      if I mod 2 = 1 then
      begin
        if I > 4 then
          break
      end
    end;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "4"]);
}

#[test]
fn break_continue_case_insensitive() {
    let out = compile_and_run(
        "\
program CaseInsensitive;
begin
  for I: integer := 1 to 10 do
  begin
    if I mod 2 = 0 then
      Continue;
    if I > 5 then
      Break;
    Std.Console.WriteLn(I)
  end
end.",
    );
    assert_eq!(out.lines, vec!["1", "3", "5"]);
}
