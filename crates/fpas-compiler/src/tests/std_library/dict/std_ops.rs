use super::*;

#[test]
fn std_dict_length() {
    let out = compile_and_run(
        "\
program DictLen;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
  Std.Console.WriteLn(Std.Dict.Length(D));
  var E: dict of string to integer := [:];
  Std.Console.WriteLn(Std.Dict.Length(E))
end.",
    );
    assert_eq!(out.lines, vec!["3", "0"]);
}

#[test]
fn std_dict_contains_key() {
    let out = compile_and_run(
        "\
program DictHas;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Std.Console.WriteLn(Std.Dict.ContainsKey(D, 'Alice'));
  Std.Console.WriteLn(Std.Dict.ContainsKey(D, 'Charlie'))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn std_dict_keys() {
    let out = compile_and_run(
        "\
program DictKeys;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Std.Console.WriteLn(Std.Dict.Keys(D))
end.",
    );
    assert_eq!(out.lines, vec!["[Alice, Bob]"]);
}

#[test]
fn std_dict_values() {
    let out = compile_and_run(
        "\
program DictVals;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Std.Console.WriteLn(Std.Dict.Values(D))
end.",
    );
    assert_eq!(out.lines, vec!["[30, 25]"]);
}

#[test]
fn std_dict_remove() {
    let out = compile_and_run(
        "\
program DictRm;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25, 'Charlie': 35];
  var D2: dict of string to integer := Std.Dict.Remove(D, 'Bob');
  Std.Console.WriteLn(D2);
  Std.Console.WriteLn(Std.Dict.Length(D2))
end.",
    );
    assert_eq!(out.lines, vec!["[Alice: 30, Charlie: 35]", "2"]);
}
