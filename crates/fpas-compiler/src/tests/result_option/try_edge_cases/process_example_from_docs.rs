use super::super::compile_and_run;
// ── Process example from docs ───────────────────────────────────────────

#[test]
fn try_process_example_ok() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(Std.Conv.IntToStr(Quotient))
end;
begin
  case Process(10, 2) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["5"]);
}

#[test]
fn try_process_example_error_propagation() {
    let out = compile_and_run(
        "program T;
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(Std.Conv.IntToStr(Quotient))
end;
begin
  case Process(10, 0) of
    Ok(V):    Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["Division by zero"]);
}
