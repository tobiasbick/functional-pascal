use super::super::*;

// ── language/basics: Comparison operators ──────────────────────────

#[test]
fn comparison_operators_all() {
    let out = compile_and_run(
        "\
program CmpOps;
uses Std.Console;
begin
  WriteLn(1 = 1);
  WriteLn(1 <> 2);
  WriteLn(1 < 2);
  WriteLn(2 > 1);
  WriteLn(1 <= 1);
  WriteLn(1 >= 1)
end.",
    );
    assert_eq!(
        out.lines,
        vec!["true", "true", "true", "true", "true", "true"]
    );
}

// ── language/basics: Logical operators on booleans ─────────────────

#[test]
fn logical_operators_full() {
    let out = compile_and_run(
        "\
program LogOps;
uses Std.Console;
begin
  WriteLn(true and true);
  WriteLn(true and false);
  WriteLn(false or true);
  WriteLn(false or false);
  WriteLn(not true);
  WriteLn(true xor false);
  WriteLn(true xor true)
end.",
    );
    assert_eq!(
        out.lines,
        vec!["true", "false", "true", "false", "false", "true", "false"]
    );
}

// ── language/basics: Bitwise operators on integers ─────────────────

#[test]
fn bitwise_and_or_not() {
    let out = compile_and_run(
        "\
program BitwiseOps;
uses Std.Console;
begin
  WriteLn(12 and 10);
  WriteLn(12 or 3);
  WriteLn(not 0)
end.",
    );
    assert_eq!(out.lines, vec!["8", "15", "-1"]);
}

#[test]
fn integer_var_from_int_div_centering() {
    let out = compile_and_run(
        "\
program Center;
uses Std.Console;
begin
  var RawX: integer := ((80 - 42) div 2) + 1;
  WriteLn(RawX)
end.",
    );
    assert_eq!(out.lines, vec!["20"]);
}

#[test]
fn real_div_of_integers_is_real() {
    let err = compile_err(
        "\
program RealDiv;
begin
  var RawX: integer := ((80 - 42) / 2) + 1
end.",
    );
    assert!(
        err.message.contains("Type mismatch"),
        "expected real `/` result to reject integer variable initializer, got: {}",
        err.message
    );
}
