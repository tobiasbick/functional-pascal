use super::compile_and_run;
#[test]
fn case_result_destructure() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(42);
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}
#[test]
fn case_result_err_branch() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('failed');
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["failed"]);
}
#[test]
fn case_option_destructure() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(99);
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}
#[test]
fn case_option_none_branch() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := None;
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["nothing"]);
}

#[test]
fn case_result_multi_label_shared_binding() {
    let out = compile_and_run(
        "program T;
var R: Result of string, string := Error('fallback');
begin
  case R of
    Ok(Msg), Error(Msg): Std.Console.WriteLn(Msg)
  end
end.",
    );
    assert_eq!(out.lines, vec!["fallback"]);
}

// ── Binding scoping (spec line 38) ──────────────────────────────────────
// "The binding variable (V, E) is scoped to its arm body."

#[test]
fn result_case_binding_not_accessible_after_case() {
    let err = super::compile_err(
        "program T;
var R: Result of integer, string := Ok(42);
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end;
  Std.Console.WriteLn(V)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}

#[test]
fn option_case_binding_not_accessible_after_case() {
    let err = super::compile_err(
        "program T;
var O: Option of integer := Some(7);
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end;
  Std.Console.WriteLn(V)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}

#[test]
fn result_error_binding_not_accessible_outside_arm() {
    let err = super::compile_err(
        "program T;
var R: Result of integer, string := Error('fail');
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end;
  Std.Console.WriteLn(E)
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_NAME);
}
