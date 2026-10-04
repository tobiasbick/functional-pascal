use super::*;

#[test]
fn fibonacci() {
    check_ok(
        r#"program Fib;

function Fibonacci(N: integer): integer;
begin
  if N <= 1 then
    return N;
  else
    return Fibonacci(N - 1) + Fibonacci(N - 2); end if;
end function;

begin
  return;
end program;"#,
    );
}

#[test]
fn calculator() {
    check_ok(
        r#"program Calculator;

type Op = enum
  OpAdd;
  OpSub;
  OpMul;
  OpDiv;
end enum;

function Calculate(A: integer; B: integer; Operation: Op): integer;
begin
  case Operation of
    when Op.OpAdd:
      return A + B;
    when Op.OpSub:
      return A - B;
    when Op.OpMul:
      return A * B;
    when Op.OpDiv:
      return A div B;
  end case;
end function;

begin
  const Answer: integer := Calculate(10, 3, Op.OpAdd);
end program;
"#,
    );
}

#[test]
fn record_usage() {
    check_ok(
        r#"program Geometry;

type Point = record
  X: real;
  Y: real;
end record;

begin
  const P: Point := Point(X := 1.0, Y := 2.0);
end program;
"#,
    );
}

#[test]
fn nested_loops_with_break_continue() {
    check_ok(
        r#"program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        if I = J then continue; end if;
        if I + J > 10 then break; end if;
      end; end for; end for;
end program;"#,
    );
}

#[test]
fn nested_mutual_recursion_even_odd() {
    check_ok(
        r#"program T;

function IsEven(N: integer): boolean;
  function IsOdd(X: integer): boolean;
  begin
    if X = 0 then return false;
    else return IsEven(X - 1); end if;
  end function;
begin
  if N = 0 then return true;
  else return IsOdd(N - 1); end if;
end function;

begin
  return;
end program;"#,
    );
}

#[test]
fn immutable_assignment_error() {
    check_errors(
        r#"program T;
 const X: integer := 42;
begin
  X := 100;
end program;"#,
    );
}

#[test]
fn mixed_errors() {
    let errs = check_errors(
        r#"program T;
begin
  break;
  const X: integer := true;
end program;"#,
    );
    assert!(errs.len() >= 2);
}
