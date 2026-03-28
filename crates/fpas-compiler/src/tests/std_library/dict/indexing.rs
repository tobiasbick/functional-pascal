use super::*;

#[test]
fn dict_index_get() {
    let out = compile_and_run(
        "\
program DictGet;
begin
  var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  Std.Console.WriteLn(D['Alice']);
  Std.Console.WriteLn(D['Bob'])
end.",
    );
    assert_eq!(out.lines, vec!["30", "25"]);
}

#[test]
fn dict_index_set_existing_key() {
    let out = compile_and_run(
        "\
program DictSetExist;
begin
  mutable var D: dict of string to integer := ['Alice': 30, 'Bob': 25];
  D['Alice'] := 31;
  Std.Console.WriteLn(D['Alice']);
  Std.Console.WriteLn(D['Bob'])
end.",
    );
    assert_eq!(out.lines, vec!["31", "25"]);
}

#[test]
fn dict_index_set_new_key() {
    let out = compile_and_run(
        "\
program DictSetNew;
begin
  mutable var D: dict of string to integer := ['Alice': 30];
  D['Bob'] := 25;
  Std.Console.WriteLn(D)
end.",
    );
    assert_eq!(out.lines, vec!["[Alice: 30, Bob: 25]"]);
}

#[test]
fn dict_key_not_found() {
    let msg = compile_run_err(
        "\
program DictMiss;
begin
  var D: dict of string to integer := ['Alice': 30];
  Std.Console.WriteLn(D['Bob'])
end.",
    );
    assert!(msg.contains("not found"), "{msg}");
}
