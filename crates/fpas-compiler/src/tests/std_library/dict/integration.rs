use super::*;

#[test]
fn dict_in_function_param_and_return() {
    let out = compile_and_run(
        "\
program DictFn;

function Greet(Ages: dict of string to integer): string;
begin
  return 'Alice is ' + Std.Conv.IntToStr(Ages['Alice'])
end;

begin
  var D: dict of string to integer := ['Alice': 30];
  Std.Console.WriteLn(Greet(D))
end.",
    );
    assert_eq!(out.lines, vec!["Alice is 30"]);
}
