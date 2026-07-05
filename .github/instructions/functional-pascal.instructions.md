---
applyTo: "**/*.fpas"
description: "Functional Pascal (FPAS) — syntax, mutability, Std library, pitfalls when editing .fpas sources. Canonical spec under docs/pascal/."
---

# Functional Pascal Language Guide

Functional Pascal is a modern Pascal dialect with immutability-by-default, first-class **named** functions, and a module system. Files use `.fpas` extension. The language is **case-insensitive** for keywords and identifiers. Authoritative documentation: [`docs/pascal/`](../../docs/pascal/). Formal syntax: [`docs/specs/grammar.ebnf`](../../docs/specs/grammar.ebnf).

## Program Structure

Programs use `program` + `begin...end.` with a trailing period. Units use `unit` and have no main block.

```pascal
program MyApp;
uses Std.Console;
begin
  WriteLn('Hello')
end.
```

```pascal
unit MyApp.Utils;
function Helper(X: integer): integer;
begin
  return X + 1
end;
```

## Types

| Type | Examples |
|------|---------|
| `integer` | `42`, `-7`, `$FF`, `1_000` |
| `real` | `3.14`, `1.5e10` (digits required both sides of `.`) |
| `boolean` | `true`, `false` |
| `char` | Must be explicitly typed |
| `string` | `'Hello'`, `'It''s'` (doubled quote), `'line1'#13#10'line2'` |
| `array of T` | `[1, 2, 3]`, `[]` — 0-based indexing |
| `dict of K to V` | `['key': value]`, `[:]` — insertion-ordered key-value map |
| `Result of T, E` | `Ok(value)`, `Error(value)` — success or error |
| `Option of T` | `Some(value)`, `None` — present or absent |

## Variables and Constants

All variables are **immutable by default**. Use `mutable var` for reassignable variables.

```pascal
var Name: string := 'Alice';
mutable var Count: integer := 0;
const MaxSize: integer := 1024;
```

## Functions and Procedures

Use `return` to return values. There is **no** `FuncName := value` return pattern.

```pascal
function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

procedure Greet(Name: string);
begin
  WriteLn('Hello, ' + Name)
end;
```

- Parameters: semicolons in declarations, commas in calls
- `mutable` parameter keyword allows reassignment inside the function
- Nested functions have lexical scope access; nest helpers for mutual recursion
- Routine headers end with `;`; bodies end with `end;`. Empty parameter lists use `()`

## Function Types and First-Class Functions

Pass **named** routines or **variables** whose type is a function/procedure type.

```pascal
type IntOp = function(A: integer; B: integer): integer;

function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value)
end;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  var R: integer := Apply(Double, 5);
  var Op: function(X: integer): integer := Double;
  WriteLn(Op(7));
  var Doubled: array of integer := Map([1, 2, 3], Double)
end.
```

## Control Flow

```pascal
if X > 0 then WriteLn('positive')
else if X = 0 then WriteLn('zero')
else WriteLn('negative');

case Value of
  1: WriteLn('one');
  2, 3: WriteLn('two or three');
  10..20: WriteLn('ten to twenty')
else
  WriteLn('other')
end;

{ Guard clauses: arm matches only when label matches AND guard is true }
case S of
  Shape.Circle(R) if R > 10.0: WriteLn('Large');
  Shape.Circle(R): WriteLn('Small');
  Shape.Rectangle(W, H): WriteLn('Rect')
end;

{ Exhaustiveness: enum/Result/Option must cover all variants or use else }
{ Guards do NOT count toward exhaustiveness }

for I: integer := 1 to 10 do WriteLn(I);
for I: integer := 10 downto 1 do WriteLn(I);
for X: integer in MyArray do WriteLn(X);

while Condition do begin ... end;
repeat ... until Done;
```

`break` and `continue` work in all loops.

## Records and Enums

```pascal
type
  Point = record
    X: real;
    Y: real;

    function DistanceTo(Self: Point; Other: Point): real;
    begin
      var DX: real := Other.X - Self.X;
      var DY: real := Other.Y - Self.Y;
      return Sqrt(DX * DX + DY * DY)
    end;
  end;

  Color = enum
    Red;
    Green;
    Blue;
  end;

var P: Point := record X := 0.0; Y := 5.0 end;
var C: Color := Color.Red;
```

Records can contain methods (functions/procedures). The first parameter is `Self` typed as the record. Callers use dot notation; `Self` is implicit: `A.DistanceTo(B)`. Field assignment requires `mutable var`.

### Enums with Associated Data

Enum variants can carry data fields (like tagged unions):

```pascal
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;

var S: Shape := Shape.Circle(5.0);
var R: Shape := Shape.Rectangle(10.0, 20.0);
```

Destructure with `case` — bindings are positional:

```pascal
case S of
  Shape.Circle(R): WriteLn('radius ' + RealToStr(R));
  Shape.Rectangle(W, H): WriteLn(RealToStr(W) + 'x' + RealToStr(H));
  Shape.Point: WriteLn('point')
end;
```

Backing values (`Success = 200`) cannot be combined with associated data on the same variant.

## Generics

Type parameters apply to **functions and procedures** (including record methods). See [`docs/pascal/language/types/generics.md`](../../docs/pascal/language/types/generics.md).

```pascal
function Identity<T>(Value: T): T;
begin
  return Value
end;

type
  Box = record
    Value: integer;

    function Map<R>(Self: Box; F: function(X: integer): R): R;
    begin
      return F(Self.Value)
    end;
  end;

var X: integer := Identity(42);  { T inferred }
```

Constraints: `<T: Comparable>`, `<T: Numeric>`, `<T: Printable>`. Details: [`docs/pascal/language/types/generics.md`](../../docs/pascal/language/types/generics.md).

## Visibility

Applies in `unit` files only (not `program` files). All declarations are **public by default**.

```pascal
unit MyApp.Render;

private function InternalHelper(X: real): real;
begin
  return X * X
end;

procedure Render(Value: real);
begin
  WriteLn(InternalHelper(Value))
end;
```

`private` hides the symbol from importers. `public` is optional/explicit. Works on `function`, `procedure`, `type`, `const`, `var`.

## Units and Uses

```pascal
program MyApp;
uses Std.Console, Std.Math, MyApp.Utils;
begin
  WriteLn(Sqrt(16.0));
  WriteLn(MyApp.Utils.Clamp(150, 0, 100))
end.
```

- Short names available after `uses` import
- Ambiguous short names require fully qualified access (e.g., `Std.Str.Length` vs `Std.Array.Length`)
- `Std.*` namespace is reserved for the standard library

## Standard Library

| Unit | Purpose | Key symbols |
|------|---------|------------|
| `Std.Console` | Text I/O, CRT screen, keyboard, events | `WriteLn`, `Write`, `ReadLn`, `ReadKey`, `ReadKeyEvent`, `KeyPressed`, `ReadEvent`, `ReadEventTimeout`, `PollEvent`, `ClrScr`, `GotoXY`, `TextColor`, `TextBackground`, `TextColorRGB`, `TextColor256`, `Window`, `Delay`, `CursorOn`, `CursorOff`, raw mode / alt screen helpers |
| `Std.Args` | Process arguments after `--` | `ParamCount`, `ParamStr` |
| `Std.Env` | Process environment | `Get`, `Exists` |
| `Std.Graph` | Native windowed graphics | Hosted: `Application.Configure`, `Application.Run`, `ApplicationHandlers`, `On*` handlers. Drawing: `Application.Open`, `Application.DrawLine`, `Application.DrawText`, `Application.Present`, `Event`, `EventKind`, … |
| `Std.Tui` | Turbo Vision terminal UI | `Application.Open`, `Create*`, `AddChild`, `OnCommand`, `Pump`, `Run`, `ExecDialog`, `RunFileDialog`, `OpenForTest`, `Test*`, … Custom paint loops belong in `Std.Console`, not `Std.Tui`. IDE About (`apps/ide`) uses custom `ExecDialog` today — see [modals.md](../../docs/pascal/std/tui/app/modals.md). |
| `Std.Str` | String operations | `Length`, `ToUpper`, `ToLower`, `Trim`, `Contains`, `StartsWith`, `EndsWith`, `Substring`, `IndexOf`, `Replace`, `Split`, `Join`, `IsNumeric`, `Format`, … |
| `Std.Conv` | Type conversions | `IntToStr`, `StrToInt`, `IntToReal`, `RealToStr`, `StrToReal`, `CharToStr`, `BoolToStr`, … |
| `Std.Math` | Math functions | `Pi`, `Sqrt`, `Pow`, `Floor`, `Ceil`, `Round`, `Sin`, `Cos`, `Log`, `Abs`, `Min`, `Max`, `Clamp`, … |
| `Std.Random` | Pseudo-random numbers | `Random`, `RandomInt`, `Randomize` |
| `Std.Array` | Array operations | `Length`, `Sort`, `Reverse`, `Contains`, `IndexOf`, `Slice`, `Push` (mutable), `Pop` (mutable), `Map`, `Filter`, `Reduce`, … |
| `Std.Dict` | Dict operations | `Length`, `ContainsKey`, `Keys`, `Values`, `Remove`, `Get`, `Merge` |
| `Std.Result` | Result helpers | `Unwrap`, `UnwrapOr`, `IsOk`, `IsError`, `Map`, `AndThen`, `OrElse` |
| `Std.Option` | Option helpers | `Unwrap`, `UnwrapOr`, `IsSome`, `IsNone`, `Map`, `AndThen`, `OrElse` |
| `Std.Task` | Task synchronization (`go`) | `Wait`, `WaitAll` |

Full TUI dispatch details: [`docs/pascal/std/tui/app/README.md`](../../docs/pascal/std/tui/app/README.md). Per-unit reference: [`docs/pascal/std/README.md`](../../docs/pascal/std/README.md).

**Ambiguous names** — always qualify these:
- `Length`, `Contains`, `IndexOf` across `Std.Str`, `Std.Array`, `Std.Dict`
- `Unwrap`, `UnwrapOr`, `Map`, `AndThen`, `OrElse` across `Std.Result` and `Std.Option`

## Operators

| Precedence | Operators |
|-----------|----------|
| 1 (highest) | `not`, unary `-`, `try` |
| 2 | `*`, `/`, `div`, `mod`, `and`, `shl`, `shr` |
| 3 | `+`, `-`, `or`, `xor` |
| 4 (lowest) | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` |

`/` always returns `real`. Use `div` for integer division.

## Semicolons

Semicolons are **separators**, not terminators. No semicolon required before `end`, `else`, `until`.

```pascal
begin
  var X: integer := 1;
  var Y: integer := 2;
  WriteLn(X + Y)         { no semicolon before end }
end
```

## Comments

```pascal
{ Brace comment }
(* Parenthesis-star comment *)
// Line comment
/// Doc line (preserved by fpas fmt on declarations)
```

Comments do **not** nest. Shared code belongs in units imported via `uses`.

## Error Handling

### Result and Option

```pascal
var R: Result of integer, string := Ok(42);
var E: Result of integer, string := Error('not found');
var O: Option of integer := Some(7);
var N: Option of integer := None;
```

Handle with `case` destructuring:

```pascal
case R of
  Ok(V):    WriteLn(IntToStr(V));
  Error(E): WriteLn('Error: ' + E)
end;
```

### Try Operator

`try` propagates errors — unwraps `Ok`/`Some`, returns early on `Error`/`None`:

```pascal
function Process(A: integer; B: integer): Result of string, string;
begin
  var Q: integer := try Divide(A, B);
  return Ok(IntToStr(Q))
end;
```

### Panic

`panic` aborts the program for unrecoverable errors:

```pascal
if B = 0 then panic('Division by zero');
```

Use `Result`/`Option` for expected failures; `panic` for broken invariants.

## Concurrency

Go-inspired **tasks** only: spawn work with `go`, synchronize with `Std.Task`. There are no channels or `select` in the current language or standard library.

```pascal
uses Std.Console, Std.Task;

function Worker(): integer;
begin
  return 42
end;

begin
  var T: task := go Worker();
  WriteLn(Wait(T))
end.
```

- `go FuncCall()` — launch a concurrent task; assign to `task` (result type follows the spawned call)
- `Wait(T)` — block until the task finishes, return its result
- `WaitAll([T1, T2, …])` — block until every task in the array completes

See [`docs/pascal/language/concurrency/README.md`](../../docs/pascal/language/concurrency/README.md).

## Projects (.fpasprj)

Multi-file projects use TOML project files. Full spec: [`docs/pascal/program-structure/projects.md`](../../docs/pascal/program-structure/projects.md).

```toml
[project]
name = "my-app"
version = "1.0.0"   # optional
kind = "program"    # or "library"
main = "src/main.fpas"

[dependencies]
projects = ["../my-lib/my-lib.fpasprj"]   # path (relative or absolute)
workspace = ["my-lib"]                      # member project.name in .fpasworkspace

[sources]
include = ["src/**/*.fpas"]
exclude = ["src/generated/**/*.fpas"]

[exports]
units = ["MyLib.Core"]   # library only: units visible to dependents
```

- **`library`** — units only, no `main`; consume via `dependencies` from a `program` project. **Source-level only** (no precompiled `.fpaslib` artifacts).
- **`fpas my-app.fpasprj`** — run a program project.
- **`fpas check`** — type-check `.fpas`, `.fpasprj`, or `.fpasworkspace` without running.
- **`.fpasworkspace`** — lists member `.fpasprj` files; `fpas check` validates all, `fpas` runs the sole program member in cwd.

Examples: [`examples/pascal/library-deps/`](../../examples/pascal/library-deps/), [`examples/pascal/monorepo/`](../../examples/pascal/monorepo/).

## Tests (`fpas test`)

- Test entry files are named `*_test.fpas` and live under [`tests/`](../../tests/) (not `examples/`).
- Run a directory, project, or single file: `fpas test tests/`, `fpas test tests/suite.fpasprj`, `fpas test tests/tui/controls/tui_turbo_vision_run_test.fpas`.
- Assertions and runner behavior: [`docs/pascal/std/testing/test.md`](../../docs/pascal/std/testing/test.md).

## Key Rules Summary

1. **Immutable by default** — use `mutable var` only when needed
2. **`return` only** — no `FuncName := value` pattern
3. **Semicolons separate** — no trailing semicolon before `end`/`else`/`until`
4. **Records support methods** — first param is `Self` typed as the record, called via dot notation
5. **Case-insensitive** — `WriteLn` = `writeln` = `WRITELN`
6. **`private` for internal symbols** — public is default in units
7. **Qualify ambiguous names** — `Std.Str.Length` vs `Std.Array.Length`; shared helpers on `Std.Result` vs `Std.Option` (`Unwrap`, `Map`, `AndThen`, `OrElse`, …)
8. **Single quotes for strings** — `'Hello'`, doubled for escaping: `'It''s'`
9. **`Result`/`Option` for expected errors** — `panic` only for broken invariants
10. **`try` propagates errors** — unwraps or returns early
11. **First-class calls** — pass named routines or function-typed variables; nest helpers for mutual recursion
12. **Graph apps use hosted dispatch** — `Application.Configure` + `Application.Run`. **Turbo Vision TUI** uses `Create*` + `OnCommand` + `Run`. **Custom terminal loops** use `Std.Console`.
