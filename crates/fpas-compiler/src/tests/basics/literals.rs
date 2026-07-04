use super::super::*;

// ── language/basics: Number Literals ───────────────────────────────

#[test]
fn hex_literal_value() {
    let out = compile_and_run(
        "\
program HexLit;
uses Std.Console;
begin
  WriteLn($FF);
  WriteLn($FF_FF)
end.",
    );
    assert_eq!(out.lines, vec!["255", "65535"]);
}

#[test]
fn underscore_literal() {
    let out = compile_and_run(
        "\
program UnderscoreLit;
uses Std.Console;
begin
  WriteLn(1_000_000)
end.",
    );
    assert_eq!(out.lines, vec!["1000000"]);
}

#[test]
fn scientific_notation_value() {
    let out = compile_and_run(
        "\
program SciNote;
uses Std.Console;
begin
  WriteLn(1.5e2);
  WriteLn(3.0E-1)
end.",
    );
    assert_eq!(out.lines, vec!["150", "0.3"]);
}

// ── language/basics: String Concatenation ──────────────────────────

#[test]
fn string_concat_with_plus() {
    let out = compile_and_run(
        "\
program StrConcat;
uses Std.Console;
begin
  var Full: string := 'Hello' + ' ' + 'World';
  WriteLn(Full)
end.",
    );
    assert_eq!(out.lines, vec!["Hello World"]);
}

#[test]
fn escaped_apostrophe_in_output() {
    let out = compile_and_run(
        "\
program EscApos;
uses Std.Console;
begin
  WriteLn('It''s Pascal')
end.",
    );
    assert_eq!(out.lines, vec!["It's Pascal"]);
}

// ── language/basics: div and mod ───────────────────────────────────

#[test]
fn div_and_mod_execution() {
    let out = compile_and_run(
        "\
program DivMod;
uses Std.Console;
begin
  WriteLn(10 div 3);
  WriteLn(10 mod 3)
end.",
    );
    assert_eq!(out.lines, vec!["3", "1"]);
}
