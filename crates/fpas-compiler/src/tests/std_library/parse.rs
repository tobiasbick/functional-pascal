use super::{compile_and_run, compile_ok};

#[test]
fn std_parse_try_int_returns_ok_for_pascal_integer_text() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Parse, Std.Result;
begin
  var R: Result of integer, string := TryInt(' +1_024 ');
  WriteLn(IsOk(R));
  WriteLn(Unwrap(R))
end.",
    );
    assert_eq!(out.lines, vec!["true", "1024"]);
}

#[test]
fn std_parse_try_real_returns_ok_for_pascal_real_text() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Parse, Std.Result;
begin
  var R: Result of real, string := Std.Parse.TryReal('1_024.0e-2');
  WriteLn(IsOk(R));
  WriteLn(Unwrap(R))
end.",
    );
    assert_eq!(out.lines, vec!["true", "10.24"]);
}

#[test]
fn std_parse_try_bool_returns_ok_for_case_insensitive_text() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Parse, Std.Result;
begin
  var R: Result of boolean, string := TryBool(' FALSE ');
  WriteLn(IsOk(R));
  WriteLn(Unwrap(R))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn std_parse_invalid_text_returns_error_without_runtime_failure() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Parse, Std.Result;
begin
  var R: Result of integer, string := TryInt('nope');
  WriteLn(IsError(R));
  WriteLn(UnwrapOr(R, 7))
end.",
    );
    assert_eq!(out.lines, vec!["true", "7"]);
}

#[test]
fn std_parse_unit_registers_for_uses() {
    compile_ok(
        "\
program T;
uses Std.Parse;
begin
  var R: Result of boolean, string := TryBool('true')
end.",
    );
}
