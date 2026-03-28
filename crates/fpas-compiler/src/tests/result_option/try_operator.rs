use super::compile_and_run;
#[test]
fn try_on_ok_unwraps() {
    let out = compile_and_run(
        "program T;
function Compute(): Result of integer, string;
begin
  return Ok(10)
end;
function Run(): Result of integer, string;
begin
  var V: integer := try Compute();
  return Ok(V + 1)
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Run()))
end.",
    );
    assert_eq!(out.lines, vec!["11"]);
}
#[test]
fn try_on_err_returns_early() {
    let out = compile_and_run(
        "program T;
function Failing(): Result of integer, string;
begin
  return Error('bad')
end;
function Run(): Result of integer, string;
begin
  var V: integer := try Failing();
  return Ok(V + 1)
end;
begin
  if Std.Result.IsError(Run()) then
    Std.Console.WriteLn('propagated')
  else
    Std.Console.WriteLn('not propagated')
end.",
    );
    assert_eq!(out.lines, vec!["propagated"]);
}
#[test]
fn try_on_some_unwraps() {
    let out = compile_and_run(
        "program T;
function MaybeVal(): Option of integer;
begin
  return Some(5)
end;
function Run(): Option of integer;
begin
  var V: integer := try MaybeVal();
  return Some(V * 2)
end;
begin
  Std.Console.WriteLn(Std.Option.Unwrap(Run()))
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}
#[test]
fn try_on_none_returns_early() {
    let out = compile_and_run(
        "program T;
function Empty(): Option of integer;
begin
  return None
end;
function Run(): Option of integer;
begin
  var V: integer := try Empty();
  return Some(V * 2)
end;
begin
  if Std.Option.IsNone(Run()) then
    Std.Console.WriteLn('none propagated')
  else
    Std.Console.WriteLn('not propagated')
end.",
    );
    assert_eq!(out.lines, vec!["none propagated"]);
}
