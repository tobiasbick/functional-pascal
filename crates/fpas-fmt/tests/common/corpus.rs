//! Valid compilation units drawn from `fpas-parser` integration and declaration tests.

/// `(label, source)` pairs that must parse and survive format → re-parse.
pub const SOURCES: &[(&str, &str)] = &[
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
        r#"program T;  var X: integer := 42; begin null; end program;"#,
    ),
    (
        "program_with_mutable_var",
        r#"program T;   mutable var Count: integer := 0; begin null; end program;"#,
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
    when OpAdd: return A + B;
    when OpSub: return A - B;
    when OpMul: return A * B;
    when OpDiv: return A div B;
  end case;
end function;

begin
  var Answer: integer := Calculate(10, 3, OpAdd);
  Console.WriteLn(Answer);
end program;"#,
    ),
    (
        "record_creation",
        r#"program Geometry;

 type Point = record
  X: real;
  Y: real;
end record;

begin
  var P: Point := record X := 1.0; Y := 2.0; end record;
  var Sum: real := P.X + P.Y;
end program;"#,
    ),
    (
        "nested_loops",
        r#"program T;
begin
  for I: integer := 0 to 9 do
    for J: integer := 0 to 9 do
      begin
        var X: integer := I * 10 + J;
        if X mod 2 = 0 then
          continue; end if;
      end; end for; end for;
end program;"#,
    ),
    (
        "repeat_with_break",
        r#"program T;
begin
  mutable var X: integer := 0;
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
  var Xs: array of integer := [1, 2, 3, 4, 5];
  var First: integer := Xs[0];
  var Last: integer := Xs[4];
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
        "record_with_method",
        r#"program T;  type Point = record X: integer; Y: integer; function Sum(Self: Point): integer; begin return Self.X + Self.Y; end function; end record; begin null; end program;"#,
    ),
    (
        "record_with_static_function",
        r#"program T;  type Point = record X: integer; Y: integer; static function Create(X: integer; Y: integer): Point; begin return record X := X; Y := Y; end record; end function; end record; begin null; end program;"#,
    ),
    (
        "record_with_static_procedure",
        r#"program T;  type Point = record X: integer; static procedure Print(Value: Point); begin Std.Console.WriteLn(Value.X); end procedure; end record; begin null; end program;"#,
    ),
    (
        "nested_collection_literals",
        r#"program T; begin var Values: array of dict of string to array of integer := [['a': [1, 2]], [:]]; end program;"#,
    ),
    (
        "nested_record_update",
        r#"program T;  type Point = record X: integer; Y: integer; end record;  type Pair = record First: Point; Second: Point; end record; begin var P: Pair := record First := record X := 1; Y := 2; end record; Second := record X := 3; Y := 4; end record; end record; var Q: Pair := P with First := P.First with X := 5; end with; end with; end program;"#,
    ),
    (
        "nested_option_result",
        r#"program T; begin var Value: result of option of array of integer, string := Ok(Some([])); end program;"#,
    ),
    (
        "case_destructure_with_guard",
        r#"program T; begin case Value of when Some(Item) if Item > 0: return; when None: return; end case; end program;"#,
    ),
    (
        "postfix_call_chain",
        r#"program T; begin return Factory.Create().Items[0].Value; end program;"#,
    ),
    (
        "procedure_literal",
        r#"program T; begin var Action: procedure() := procedure() begin return; end procedure; end program;"#,
    ),
];
