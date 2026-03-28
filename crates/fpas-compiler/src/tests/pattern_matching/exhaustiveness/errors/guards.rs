use super::*;

#[test]
fn exhaustiveness_guarded_arm_does_not_count() {
    let err = compile_err(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Red;
  case L of
    Light.Red if true: Std.Console.WriteLn('stop');
    Light.Yellow: Std.Console.WriteLn('caution');
    Light.Green: Std.Console.WriteLn('go')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Red"));
}

#[test]
fn exhaustiveness_all_guarded_data_enum() {
    let err = compile_err(
        "\
program T;
type Val = enum Num(N: integer); Text(S: string); end;
begin
  var V: Val := Val.Num(1);
  case V of
    Val.Num(N) if N > 0: Std.Console.WriteLn('pos');
    Val.Text(S) if true: Std.Console.WriteLn(S)
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Num"));
    assert!(err.message.contains("Text"));
}

#[test]
fn exhaustiveness_error_single_variant_enum_no_arms() {
    let err = compile_err(
        "\
program T;
type Wrap = enum Val(N: integer); end;
begin
  var W: Wrap := Wrap.Val(1);
  case W of
    Wrap.Val(N) if N > 0: Std.Console.WriteLn('pos')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Val"));
}
