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
        "program T; var X: integer := 42; begin end.",
    ),
    (
        "program_with_mutable_var",
        "program T; mutable var Count: integer := 0; begin end.",
    ),
    (
        "hello_world",
        "program Hello;\nuses\n  Std.Console;\nbegin\n  Std.Console.WriteLn('Hello, World!')\nend.",
    ),
    (
        "calculator",
        "program Calculator;\nuses Std.Console;\n\ntype Op = enum\n  OpAdd;\n  OpSub;\n  OpMul;\n  OpDiv;\nend;\n\nfunction Calculate(A: integer; B: integer; Operation: Op): integer;\nbegin\n  case Operation of\n    OpAdd: return A + B;\n    OpSub: return A - B;\n    OpMul: return A * B;\n    OpDiv: return A div B\n  end\nend;\n\nbegin\n  var Answer: integer := Calculate(10, 3, OpAdd);\n  Std.Console.WriteLn(Answer)\nend.",
    ),
    (
        "record_creation",
        "program Geometry;\n\ntype Point = record\n  X: real;\n  Y: real;\nend;\n\nbegin\n  var P: Point := record X := 1.0; Y := 2.0; end;\n  var Sum: real := P.X + P.Y\nend.",
    ),
    (
        "nested_loops",
        "program T;\nbegin\n  for I: integer := 0 to 9 do\n    for J: integer := 0 to 9 do\n      begin\n        var X: integer := I * 10 + J;\n        if X mod 2 = 0 then\n          continue\n      end\nend.",
    ),
    (
        "repeat_with_break",
        "program T;\nbegin\n  mutable var X: integer := 0;\n  repeat\n    X := X + 1;\n    if X = 10 then break\n  until X = 100\nend.",
    ),
    (
        "array_operations",
        "program T;\nbegin\n  var Xs: array of integer := [1, 2, 3, 4, 5];\n  var First: integer := Xs[0];\n  var Last: integer := Xs[4]\nend.",
    ),
    (
        "fibonacci",
        "program Fib;\nuses Std.Console;\n\nfunction Fibonacci(N: integer): integer;\nbegin\n  if N <= 1 then\n    return N\n  else\n    return Fibonacci(N - 1) + Fibonacci(N - 2)\nend;\n\nbegin\n  Std.Console.WriteLn(Fibonacci(10))\nend.",
    ),
    (
        "nested_mutual_recursion",
        "program T;\n\nfunction IsEven(N: integer): boolean;\n  function IsOdd(X: integer): boolean;\n  begin\n    if X = 0 then return false\n    else return IsEven(X - 1)\n  end;\nbegin\n  if N = 0 then return true\n  else return IsOdd(N - 1)\nend;\n\nbegin\n  return\nend.",
    ),
    (
        "unit_clamp_compact",
        "unit MyApp.Utils; uses Std.Math; function Clamp(Value: integer; Min: integer; Max: integer): integer; begin if Value < Min then return Min else if Value > Max then return Max else return Value end; function IsBlank(S: string): boolean; begin return Length(Trim(S)) = 0 end;",
    ),
    (
        "unit_private_routine",
        "unit MyApp.Utils; function Clamp(Value: integer): integer; begin return Value end; private function Hidden(): integer; begin return 0 end;",
    ),
    (
        "enum_type",
        "program T; type Color = enum Red; Green; Blue; end; begin end.",
    ),
    (
        "record_with_method",
        "program T; type Point = record X: integer; Y: integer; function Sum(Self: Point): integer; begin return Self.X + Self.Y end; end; begin end.",
    ),
];
