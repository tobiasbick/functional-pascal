use super::*;

#[test]
fn std_conv() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Conv.IntToStr(42));
  Std.Console.WriteLn(Std.Conv.StrToInt('7'));
  Std.Console.WriteLn(Std.Conv.IntToReal(3));
  Std.Console.WriteLn(Std.Conv.RealToStr(1.5));
  Std.Console.WriteLn(Std.Conv.StrToReal('2.25'));
  Std.Console.WriteLn(Std.Conv.CharToStr('Z'))
end.",
    );
    assert_eq!(out.lines, vec!["42", "7", "3", "1.5", "2.25", "Z"]);
}

#[test]
fn std_str_to_int_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var N: integer := Std.Conv.StrToInt('x')
end.",
    );
    assert!(msg.contains("StrToInt") || msg.contains("invalid"), "{msg}");
}

#[test]
fn std_str_to_real_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
begin
  var R: real := Std.Conv.StrToReal('not-a-number')
end.",
    );
    assert!(
        msg.contains("StrToReal") || msg.contains("invalid"),
        "{msg}"
    );
}

#[test]
fn std_str_to_real_rejects_non_pascal_real_text() {
    let msg = compile_run_err(
        "\
program T;
begin
  var R: real := Std.Conv.StrToReal('NaN')
end.",
    );
    assert!(
        msg.contains("StrToReal") || msg.contains("invalid real"),
        "{msg}"
    );
}

#[test]
fn std_str_to_real_accepts_trimmed_scientific_pascal_text() {
    let out = compile_and_run(
        "\
program T;
begin
  Std.Console.WriteLn(Std.Conv.StrToReal('  +1_024.0e-2  '))
end.",
    );
    assert_eq!(out.lines, vec!["10.24"]);
}

#[test]
fn std_str_to_real_rejects_missing_fraction_digits() {
    let msg = compile_run_err(
        "\
program T;
begin
  var R: real := Std.Conv.StrToReal('5.')
end.",
    );
    assert!(
        msg.contains("StrToReal") || msg.contains("invalid real"),
        "{msg}"
    );
}
