use super::super::{compile_and_run, compile_err};
// ── Multiple try in one function ────────────────────────────────────────

#[test]
fn try_chained_multiple_calls() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Ok(10)
end;
function GetB(): Result of integer, string;
begin
  return Ok(20)
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  Std.Console.WriteLn(Std.Result.Unwrap(Combined()))
end.",
    );
    assert_eq!(out.lines, vec!["30"]);
}

#[test]
fn try_chained_first_fails() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Error('A failed')
end;
function GetB(): Result of integer, string;
begin
  return Ok(20)
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  case Combined() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["A failed"]);
}

#[test]
fn try_chained_second_fails() {
    let out = compile_and_run(
        "program T;
function GetA(): Result of integer, string;
begin
  return Ok(10)
end;
function GetB(): Result of integer, string;
begin
  return Error('B failed')
end;
function Combined(): Result of integer, string;
begin
  var A: integer := try GetA();
  var B: integer := try GetB();
  return Ok(A + B)
end;
begin
  case Combined() of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["B failed"]);
}
