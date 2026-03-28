use super::*;

#[test]
fn if_with_begin_end_block() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 15;
  if X > 10 then
  begin
    Std.Console.WriteLn('large');
    Std.Console.WriteLn('indeed')
  end
end.",
    );
    assert_eq!(out.lines, vec!["large", "indeed"]);
}

#[test]
fn if_else_with_begin_end_blocks() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 3;
  if X > 10 then
  begin
    Std.Console.WriteLn('large');
    Std.Console.WriteLn('number')
  end
  else
  begin
    Std.Console.WriteLn('small');
    Std.Console.WriteLn('number')
  end
end.",
    );
    assert_eq!(out.lines, vec!["small", "number"]);
}

#[test]
fn if_block_with_mutable_var() {
    let out = compile_and_run(
        "\
program T;
begin
  mutable var X: integer := 15;
  if X > 10 then
  begin
    Std.Console.WriteLn('large');
    X := X - 10
  end
  else
  begin
    Std.Console.WriteLn('small')
  end;
  Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["large", "5"]);
}
