use super::*;

#[test]
fn dict_literal() {
    let out = compile_and_run(
        "\
program DictLit;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Std.Console.WriteLn(D)
end.",
    );
    assert_eq!(out.lines, vec!["[Alice: 30, Bob: 25]"]);
}

#[test]
fn dict_empty() {
    let out = compile_and_run(
        "\
program DictEmpty;
begin
  var D: dict of string to integer := [:];
  Std.Console.WriteLn(D)
end.",
    );
    assert_eq!(out.lines, vec!["[]"]);
}

#[test]
fn dict_integer_keys() {
    let out = compile_and_run(
        "\
program DictIntKey;
begin
  var D: dict of integer to string := [1: 'one', 2: 'two'];
  Std.Console.WriteLn(D[1]);
  Std.Console.WriteLn(D[2])
end.",
    );
    assert_eq!(out.lines, vec!["one", "two"]);
}

#[test]
fn dict_single_entry() {
    let out = compile_and_run(
        "\
program DictOne;
begin
  var D: dict of string to boolean := ['flag': true];
  Std.Console.WriteLn(D['flag']);
  Std.Console.WriteLn(Std.Dict.Length(D))
end.",
    );
    assert_eq!(out.lines, vec!["true", "1"]);
}
