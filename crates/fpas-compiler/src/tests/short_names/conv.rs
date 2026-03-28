use super::super::*;

#[test]
fn short_conv_all() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(42));
  WriteLn(StrToInt('7'));
  WriteLn(IntToReal(3));
  WriteLn(RealToStr(1.5));
  WriteLn(StrToReal('2.25'));
  WriteLn(CharToStr('Z'))
end.",
    );
    assert_eq!(out.lines, vec!["42", "7", "3", "1.5", "2.25", "Z"]);
}

#[test]
fn short_conv_runtime_error() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Conv;
begin
  var N: integer := StrToInt('not_a_number')
end.",
    );
    assert!(msg.contains("StrToInt") || msg.contains("invalid"), "{msg}");
}
