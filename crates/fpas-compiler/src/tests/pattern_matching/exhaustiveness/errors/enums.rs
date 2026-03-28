use super::*;

#[test]
fn exhaustiveness_error_missing_enum_variant() {
    let err = compile_err(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Red;
  case L of
    Light.Red: Std.Console.WriteLn('stop');
    Light.Green: Std.Console.WriteLn('go')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Yellow"));
}

#[test]
fn exhaustiveness_error_missing_multiple_enum_variants() {
    let err = compile_err(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Red;
  case L of
    Light.Red: Std.Console.WriteLn('stop')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Yellow"));
    assert!(err.message.contains("Green"));
}
