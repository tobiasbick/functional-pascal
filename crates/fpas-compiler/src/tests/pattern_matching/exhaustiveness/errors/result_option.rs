use super::*;

#[test]
fn exhaustiveness_error_missing_result_err() {
    let err = compile_err(
        "\
program T;
begin
  var R: Result of integer, string := Ok(1);
  case R of
    Ok(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Error"));
}

#[test]
fn exhaustiveness_error_missing_result_ok() {
    let err = compile_err(
        "\
program T;
begin
  var R: Result of integer, string := Error('x');
  case R of
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Ok"));
}

#[test]
fn exhaustiveness_error_missing_result_both() {
    let err = compile_err(
        "\
program T;
begin
  var R: Result of integer, string := Ok(1);
  case R of
    Ok(V) if V > 0: Std.Console.WriteLn('pos')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Ok"));
    assert!(err.message.contains("Error"));
}

#[test]
fn exhaustiveness_error_missing_option_none() {
    let err = compile_err(
        "\
program T;
begin
  var O: Option of integer := Some(1);
  case O of
    Some(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("None"));
}

#[test]
fn exhaustiveness_error_missing_option_some() {
    let err = compile_err(
        "\
program T;
begin
  var O: Option of integer := None;
  case O of
    None: Std.Console.WriteLn('absent')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Some"));
}

#[test]
fn exhaustiveness_error_missing_option_both() {
    let err = compile_err(
        "\
program T;
begin
  var O: Option of integer := Some(1);
  case O of
    Some(V) if V > 0: Std.Console.WriteLn('pos')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Some"));
    assert!(err.message.contains("None"));
}
