//! Valid compilation units drawn from `fpas-parser` integration and declaration tests.

/// `(label, source)` pairs that must parse and survive format → re-parse.
pub const SOURCES: &[(&str, &str)] = &[
    ("minimal_program", "program Hello; begin end."),
    (
        "program_with_uses",
        "program Test; uses Std.Console, Std.Math; begin end.",
    ),
    (
        "program_with_const",
        "program T; const Pi: real := 3.14; begin end.",
    ),
    (
        "program_with_var",
        "program T; const X: integer := 42; begin end.",
    ),
    (
        "program_with_mutable_var",
        "program T; var Count: integer := 0; begin end.",
    ),
    (
        "hello_world",
        "program Hello;\nuses\n  Std.Console;\nbegin\n  Std.Console.WriteLn('Hello, World!');\nend.",
    ),
    (
        "calculator",
        "program Calculator;\nuses Std.Console;\n\ntype Op = enum\n  OpAdd;\n  OpSub;\n  OpMul;\n  OpDiv;\nend enum;\n\nfunction Calculate(A: integer; B: integer; Operation: Op): integer;\nbegin\n  case Operation of\n    when OpAdd: return A + B;\n    when OpSub: return A - B;\n    when OpMul: return A * B;\n    when OpDiv: return A div B;\n  end case;\nend function;\n\nbegin\n  const Answer: integer := Calculate(10, 3, OpAdd);\n  Std.Console.WriteLn(Answer);\nend.",
    ),
    (
        "record_creation",
        "program Geometry;\n\ntype Point = record\n  X: real;\n  Y: real;\nend record;\n\nbegin\n  const P: Point := Point( X := 1.0, Y := 2.0 );\n  const Sum: real := P.X + P.Y;\nend.",
    ),
    (
        "nested_loops",
        "program T;\nbegin\n  for I: integer := 0 to 9 do\n    for J: integer := 0 to 9 do\n      begin\n        const X: integer := I * 10 + J;\n        if X mod 2 = 0 then\n          continue; end if;\n      end; end for; end for;\nend.",
    ),
    (
        "repeat_with_break",
        "program T;\nbegin\n  var X: integer := 0;\n  repeat\n    X := X + 1;\n    if X = 10 then break; end if;\n  until X = 100;\nend.",
    ),
    (
        "array_operations",
        "program T;\nbegin\n  const Xs: array of integer := [1, 2, 3, 4, 5];\n  const First: integer := Xs[0];\n  const Last: integer := Xs[4];\nend.",
    ),
    (
        "fibonacci",
        "program Fib;\nuses Std.Console;\n\nfunction Fibonacci(N: integer): integer;\nbegin\n  if N <= 1 then\n    return N;\n  else\n    return Fibonacci(N - 1) + Fibonacci(N - 2); end if;\nend function;\n\nbegin\n  Std.Console.WriteLn(Fibonacci(10));\nend.",
    ),
    (
        "nested_mutual_recursion",
        "program T;\n\nfunction IsEven(N: integer): boolean;\n  function IsOdd(X: integer): boolean;\n  begin\n    if X = 0 then return false;\n    else return IsEven(X - 1); end if;\n  end function;\nbegin\n  if N = 0 then return true;\n  else return IsOdd(N - 1); end if;\nend function;\n\nbegin\n  return;\nend.",
    ),
    (
        "unit_clamp_compact",
        "unit MyApp.Utils; uses Std.Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min; elsif  Value > Max then return Max; else return Value; end if; end function; function IsBlank(S: string): boolean; begin return S.Trim().Length() = 0; end function;\nend unit;",
    ),
    (
        "unit_mixed_visibility",
        "unit MyApp.Utils; public function Clamp(Value: integer): integer; begin return Value; end function; function Hidden(): integer; begin return 0; end function;\nend unit;",
    ),
    (
        "enum_type",
        "program T; type Color = enum Red; Green; Blue; end enum; begin end.",
    ),
    (
        "record_with_method",
        "program T; type Point = record X: integer; Y: integer; function Sum(Self: Point): integer; begin return Self.X + Self.Y; end function; end record; begin end.",
    ),
    (
        "record_with_static_function",
        "program T; type Point = record X: integer; Y: integer; static function Create(X: integer; Y: integer): Point; begin return Point( X := X, Y := Y ); end function; end record; begin end.",
    ),
    (
        "record_with_static_procedure",
        "program T; type Point = record X: integer; static procedure Print(Value: Point); begin Std.Console.WriteLn(Value.X); end procedure; end record; begin end.",
    ),
    (
        "nested_collection_literals",
        "program T; begin const Values: array of dict of string to array of integer := [['a': [1, 2]], [:]]; end.",
    ),
    (
        "nested_record_update",
        "program T; type Point = record X: integer; Y: integer; end record; type Pair = record First: Point; Second: Point; end record; begin const P: Pair := Pair( First := Point( X := 1, Y := 2 ), Second := Point( X := 3, Y := 4 ) ); const Q: Pair := P with First := P.First with X := 5; end with; end with; end.",
    ),
    (
        "nested_option_result",
        "program T; begin const Value: result of (option of array of integer, string) := Ok(Some([])); end.",
    ),
    (
        "case_destructure_with_guard",
        "program T; begin case Value of when Some(const Item) if Item > 0: return; when None: return; end case; end.",
    ),
    (
        "postfix_call_chain",
        "program T; begin return Factory.Create().Items[0].Value; end.",
    ),
    (
        "procedure_literal",
        "program T; begin const Action: procedure() := procedure() begin return; end procedure; end.",
    ),
];
