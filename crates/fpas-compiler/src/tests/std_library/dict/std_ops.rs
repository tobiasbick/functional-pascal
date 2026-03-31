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
  Std.Console.WriteLn(Std.Dict.Length(D2));
  Std.Console.WriteLn(Std.Dict.ContainsKey(D2, 'Bob'));
  Std.Console.WriteLn(Std.Dict.Length(D))
end.",
    );
    assert_eq!(out.lines, vec!["2", "false", "3"]);
}

// ── Dict.Map ──────────────────────────────────────────────────────────────────

#[test]
fn std_dict_map_doubles_values() {
    let out = compile_and_run(
        "\
program DictMapDouble;
function Double(V: integer): integer;
begin
  return V * 2
end;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3];
  var D2: dict of string to integer := Std.Dict.Map(D, Double);
  Std.Console.WriteLn(D2)
end.",
    );
    assert_eq!(out.lines, vec!["[A: 2, B: 4, C: 6]"]);
}

#[test]
fn std_dict_map_preserves_keys() {
    let out = compile_and_run(
        "\
program DictMapKeys;
function AddFive(V: integer): integer;
begin
  return V + 5
end;
begin
  var D: dict of string to integer := ['X': 10, 'Y': 20];
  var D2: dict of string to integer := Std.Dict.Map(D, AddFive);
  Std.Console.WriteLn(Std.Dict.Keys(D2))
end.",
    );
    assert_eq!(out.lines, vec!["[X, Y]"]);
}

#[test]
fn std_dict_map_empty_dict_returns_empty() {
    let out = compile_and_run(
        "\
program DictMapEmpty;
function Mul99(V: integer): integer;
begin
  return V * 99
end;
begin
  var D: dict of string to integer := [:];
  var D2: dict of string to integer := Std.Dict.Map(D, Mul99);
  Std.Console.WriteLn(Std.Dict.Length(D2))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_dict_map_wrong_arg_count_is_compile_error() {
    let err = compile_err(
        "\
program DictMapBad;
begin
  var D: dict of string to integer := ['A': 1];
  var _: dict of string to integer := Std.Dict.Map(D)
end.",
    );
    let msg = format!("{err:?}");
    assert!(msg.contains("Map") || msg.contains("argument"), "{msg}");
}

// ── Dict.Filter ───────────────────────────────────────────────────────────────

#[test]
fn std_dict_filter_keeps_matching_entries() {
    let out = compile_and_run(
        "\
program DictFilter;
function GreaterThanTwo(K: string; V: integer): boolean;
begin
  return V > 2
end;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2, 'C': 3, 'D': 4];
  var Big: dict of string to integer := Std.Dict.Filter(D, GreaterThanTwo);
  Std.Console.WriteLn(Big)
end.",
    );
    assert_eq!(out.lines, vec!["[C: 3, D: 4]"]);
}

#[test]
fn std_dict_filter_all_pass() {
    let out = compile_and_run(
        "\
program DictFilterAll;
function AlwaysTrue(K: string; V: integer): boolean;
begin
  return true
end;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2];
  var All: dict of string to integer := Std.Dict.Filter(D, AlwaysTrue);
  Std.Console.WriteLn(Std.Dict.Length(All))
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn std_dict_filter_none_pass() {
    let out = compile_and_run(
        "\
program DictFilterNone;
function AlwaysFalse(K: string; V: integer): boolean;
begin
  return false
end;
begin
  var D: dict of string to integer := ['A': 1, 'B': 2];
  var None_: dict of string to integer := Std.Dict.Filter(D, AlwaysFalse);
  Std.Console.WriteLn(Std.Dict.Length(None_))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_dict_filter_empty_dict() {
    let out = compile_and_run(
        "\
program DictFilterEmpty;
function Positive(K: string; V: integer): boolean;
begin
  return V > 0
end;
begin
  var D: dict of string to integer := [:];
  var F: dict of string to integer := Std.Dict.Filter(D, Positive);
  Std.Console.WriteLn(Std.Dict.Length(F))
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_dict_filter_uses_key_in_predicate() {
    let out = compile_and_run(
        "\
program DictFilterKey;
uses Std.Str;
function StartsWithA(K: string; V: integer): boolean;
begin
  return Std.Str.StartsWith(K, 'a')
end;
begin
  var D: dict of string to integer := ['apple': 1, 'banana': 2, 'apricot': 3];
  var A: dict of string to integer := Std.Dict.Filter(D, StartsWithA);
  Std.Console.WriteLn(Std.Dict.Length(A))
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn std_dict_filter_wrong_arg_count_is_compile_error() {
    let err = compile_err(
        "\
program DictFilterBad;
begin
  var D: dict of string to integer := ['A': 1];
  var _: dict of string to integer := Std.Dict.Filter(D)
end.",
    );
    let msg = format!("{err:?}");
    assert!(msg.contains("Filter") || msg.contains("argument"), "{msg}");
}
