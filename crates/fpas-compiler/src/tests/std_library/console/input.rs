use super::*;

#[test]
fn std_readln_write() {
    let out = compile_run_with_readln(
        "\
program T;
begin
  var S: string := Std.Console.ReadLn();
  Std.Console.WriteLn(S)
end.",
        &["alpha"],
    );
    assert_eq!(out.lines, vec!["alpha"]);
}

#[test]
fn std_console_read_and_readln_share_line_buffer() {
    let out = compile_run_with_readln(
        "\
program T;
uses Std.Console;
begin
  Std.Console.WriteLn(Std.Console.Read());
  Std.Console.WriteLn(Std.Console.ReadLn());
end.",
        &["hello"],
    );
    assert_eq!(out.lines, vec!["h", "ello"]);
}

#[test]
fn std_console_keypressed_and_readkey() {
    let out = compile_run_with_readln_and_readkey(
        "\
program T;
uses Std.Console;
begin
  Std.Console.WriteLn(Std.Console.KeyPressed());
  Std.Console.WriteLn(Std.Console.ReadKey());
  Std.Console.WriteLn(Std.Console.KeyPressed());
end.",
        &[],
        "z",
    );
    assert_eq!(out.lines, vec!["true", "z", "false"]);
}
