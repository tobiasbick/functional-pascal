//! Valid compilation units drawn from `fpas-parser` integration and declaration tests.

/// `(label, source)` pairs that must parse and survive format → re-parse.
pub const SOURCES: &[(&str, &str)] = &[
    (
        "explicit_var_modes",
        "program T; type Action = procedure(var Value: integer); procedure Change(var Value: integer); begin Value := Value + 1; end procedure; begin var Items: array of (integer) := [1]; const F: Action := Change; F(var Items[0]); end program;",
    ),
    (
        "minimal_program",
        r#"program Hello; begin null; end program;"#,
    ),
    (
        "program_with_uses",
        r#"program Test;  uses Std.Console as Console; uses Std.Math as Math; begin null; end program;"#,
    ),
    (
        "program_with_const",
        r#"program T;  const Pi: real := 3.14; begin null; end program;"#,
    ),
    (
        "program_with_var",
        r#"program T;  const X: integer := 42; begin null; end program;"#,
    ),
    (
        "program_with_mutable_var",
        r#"program T;    var Count: integer := 0; begin null; end program;"#,
    ),
    (
        "hello_world",
        r#"program Hello;

  uses Std.Console as Console;
begin
  Console.WriteLn('Hello, World!');
end program;"#,
    ),
    (
        "calculator",
        r#"program Calculator;

uses Std.Console as Console;

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
  Console.WriteLn(Answer);
end program;
"#,
    ),
    (
        "record_creation",
        r#"program Geometry;

type Point = record
  X: real;
  Y: real;
end record;

begin
  const P: Point := Point(X := 1.0, Y := 2.0);
  const Sum: real := P.X + P.Y;
end program;
"#,
    ),
    (
        "nested_loops",
        r#"program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        const X: integer := I * 10 + J;
        if X mod 2 = 0 then
          continue; end if;
      end; end for; end for;
end program;"#,
    ),
    (
        "repeat_with_break",
        r#"program T;
begin
   var X: integer := 0;
  repeat
    X := X + 1;
    if X = 10 then break; end if;
  until X = 100;
end program;"#,
    ),
    (
        "array_operations",
        r#"program T;
begin
  const Xs: array of (integer) := [1, 2, 3, 4, 5];
  const First: integer := Xs[0];
  const Last: integer := Xs[4];
end program;"#,
    ),
    (
        "fibonacci",
        r#"program Fib;
 uses Std.Console as Console;

function Fibonacci(N: integer): integer;
begin
  if N <= 1 then
    return N;
  else
    return Fibonacci(N - 1) + Fibonacci(N - 2); end if;
end function;

begin
  Console.WriteLn(Fibonacci(10));
end program;"#,
    ),
    (
        "nested_mutual_recursion",
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
    ),
    (
        "unit_clamp_compact",
        r#"unit MyApp.Utils;  uses Std.Math as Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min; else if Value > Max then return Max; else return Value; end if; end if; end function; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0; end function;
end unit;
"#,
    ),
    (
        "unit_mixed_visibility",
        r#"unit MyApp.Utils; public function Clamp(Value: integer): integer; begin return Value; end function; function Hidden(): integer; begin return 0; end function;
end unit;
"#,
    ),
    (
        "enum_type",
        r#"program T;  type Color = enum Red; Green; Blue; end enum; begin null; end program;"#,
    ),
    (
        "record_function",
        r#"program T; type Point = record X: integer; Y: integer; end record; function PointSum(Receiver: Point): integer; begin return Receiver.X + Receiver.Y; end function; begin null; end program;"#,
    ),
    (
        "record_factory",
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

  function PointCreate(X: integer; Y: integer): Point;
  begin
    return Point(X := X, Y := Y);
  end function;

begin
  null;
end program;
"#,
    ),
    (
        "record_procedure",
        r#"program T; uses Std.Console as Console; type Point = record X: integer; end record; procedure PointPrint(Value: Point); begin Console.WriteLn(Value.X); end procedure; begin null; end program;"#,
    ),
    (
        "nested_collection_literals",
        r#"program T; begin const Values: array of (dict of (string, array of (integer))) := [['a': [1, 2]], [:]]; end program;"#,
    ),
    (
        "nested_record_update",
        r#"program T;

type Point = record
  X: integer;
  Y: integer;
end record;

type Pair = record
  First: Point;
  Second: Point;
end record;

begin
  const P: Pair := Pair(First := Point(X := 1, Y := 2), Second := Point(X := 3, Y := 4));
  const Q: Pair := P with First := P.First with X := 5; end with; end with;
end program;
"#,
    ),
    (
        "nested_option_result",
        r#"program T; begin const Value: result of (option of (array of (integer)), string) := Result.Ok(Option.Some([])); end program;"#,
    ),
    (
        "case_destructure_with_guard",
        r#"program T; begin case Value of when Option.Some(const Item) if Item > 0: return; when Option.None: return; end case; end program;"#,
    ),
    (
        "postfix_call_chain",
        r#"program T; begin return Factory.Create().Items[0].Value; end program;"#,
    ),
    (
        "procedure_literal",
        r#"program T; begin const Action: procedure() := procedure() begin return; end procedure; end program;"#,
    ),
];
