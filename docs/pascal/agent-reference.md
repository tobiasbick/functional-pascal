# Compact FPAS language reference

Read this file first when writing Functional Pascal. It covers the implemented
language in one place; follow the topic links for detailed rules and library APIs.
The [formal grammar](../specs/grammar.ebnf) describes syntax. The
[handbook](README.md) explains behavior. Planned features belong in `docs/future/`.
Purpose and upkeep are recorded in the [maintenance guide](agent-reference-maintenance.md).

The editor command **Functional Pascal: Show Public Signatures** provides a read-only,
source-derived view of the current unit's public declarations, including unsaved edits.

Every fenced example below is a complete program or unit. The CLI test
`agent_reference_examples_check_and_run` extracts all fences, checks every source,
and runs every program, including its assertions. Save a standalone program as a
`.fpas` file and use `fpas run file.fpas`; the unit example describes its project setup.
Tests import `Std.Test` explicitly. It is a normal library unit, not implicit syntax.

## 1. Translate habits before writing

Keywords and ASCII identifiers are case-insensitive. Prefer the formatter's casing and
spacing. FPAS has values, records, functions and managed tasks; it has no classes,
inheritance, pointers, manual allocation, exceptions with handlers, or preprocessor.

| Familiar Pascal / Delphi / Ada habit | FPAS spelling or rule |
| --- | --- |
| A `var` or `type` section containing several declarations | Repeat `var`, `const`, or `type` for each declaration. |
| Pascal `const N = 3` | `const N: integer := 3;` (locals may infer known types). |
| Ada `X : Integer := 1` | `var X: integer := 1;`; use `const` for an immutable binding. |
| Pascal `var I, J: integer;` or a binding without value | One name per declaration, always initialized: `var I: integer := 0;`. |
| Pascal `var` section between a routine header and `begin` | Declare locals as `var`/`const` statements after `begin`. |
| Pascal `array[0..2] of T = (A, B, C)` | `array of T := [A, B, C]`; arrays have no bounds and start at 0. |
| Delphi `class`, `uses SysUtils` | A `record` with methods taking `Self`; import `Std` units with an alias. |
| Pascal `Inc(X)`, `Low(A)`, `High(A)`, `nil` | `X := X + 1;`, `0`, `A.Length() - 1`, `None`. |
| Delphi `Double`, `Int64`, `Write(Text)` | `real`, `integer`, `Console.WriteText(Text)`. |
| Assign to the function name, Delphi `Result` or `Exit(Value)` | `return Value;`; procedures use `return;`. |
| Delphi `const` parameters | Parameters are read-only already: `function F(S: string): integer;`. |
| Pascal `procedure Q;` without parentheses | Every routine declares its parameter list: `procedure Q();`. |
| Ada `function F return T is ... end F;` | `function F(): T; begin ... end function;`. |
| Ada `program Main is ... end Main;`, `type T is ...`, `in out` | `program Main; ... end.`, `type T = ...`, `var` parameters. |
| Ada `&`, `Integer'Image (N)`, `(1, 2)` aggregates | `+`, `Conv.IntToStr(N)`, `[1, 2]` or `Point(X := 1, Y := 2)`. |
| Pascal `type K = (A, B)` | `type K = enum A; B; end enum;`. |
| Pascal `end;` for every block | `end function;`, `end procedure;`, `end if;`, etc.; program `end.`. |
| Omit the semicolon before Pascal `else` | Terminate each statement with `;`, including before `else`. |
| Ada `elsif` | Also `elsif`; nested `else if` requires a second `end if`. |
| Pascal `case X of 1: ...` | `case X of when 1: ... end case;`. |
| Ada `for I in 1 .. 3 loop` | `for I: integer := 1 to 3 do ... end for;`. |
| Pascal `for I := 1 to N do`, `begin ... end` as a body | Typed loop variable; the body is a statement list closed by `end for;`. |
| Delphi `T.Create(...)`, positional record initialization | `Point(X := 1, Y := 2)`; record fields are named. |
| Mutating `Self` or `this` | `Self` is read-only; return an updated record. |
| Ada named actuals `X => 1` | `X := 1`; an argument list is entirely named or entirely positional. |
| Pascal `var` parameter passed as `F(X)` | Declare `var X: T`, call `F(var X)` (or `F(X := var X)`). |
| Pascal routine address `@F` | Use `F` as the callable value and `F(...)` to call. |
| C-family `==`, `!=`, `&&`, `\|\|`, `!`, `"text"`; Ada `/=` | `=`, `<>`, `and`, `or`, `not`, `'text'`; `<>`. |
| Pascal `WriteLn(X:0:2)` | No width specifiers; format first, for example `'%f'.Format(X)`. |
| Delphi `TList<Integer>` | `array of integer`; generic types use `Box of integer`. |
| Pascal one-based strings / `Char` | Zero-based Unicode scalar indexing; a character is a `string`. |
| Pascal `{comment}` / `(*comment*)`, Ada `--comment` | Only `//` comments. |
| Pascal bitwise `and`, `or`, `xor` on integers | These operators take booleans; use `Std.Bits` for bits. |
| Delphi `try` / `except`, Ada `exception when`, `raise` | Return `Result of (T, E)` or `Option of T`; `try Expr` propagates failure; `panic` stops on a bug. |
| Ada `with Ada.Text_IO; use Ada.Text_IO;`, `procedure Main is` | `program Main;`, then `uses Std.Console as Console;`. |
| Ada `task` declarations | Start a routine with `go Work()` inside `scope ... end scope;`; channels and `Wait` come from `Std.Tasks`. |
| Delphi `StrToInt(Text)` / `StrToFloat(Text)` | Import `Std.Parse`; use `Parse.Int(Text)` / `Parse.Real(Text)` and handle the Result, or explicitly Unwrap. |
| Delphi `interface` / `implementation`, Ada package spec/body | One `unit Name; ... end unit;`; mark each export `public`. |
| An imported unit is automatically available through a dependency | Add `uses`; project dependencies only make sources discoverable. |

## 2. Programs, literals, bindings, and operators

A program has a header, optional `uses`, top-level declarations, then `begin ... end.`.
Types are declared at top level. Local `const` and `var` declarations appear inline
in statement lists and have lexical scope. A nested `begin ... end;` introduces a
block. Parameters and loop variables are read-only; introduce a local `var` to change
a copy. `const` initializers may run at runtime: immutability does not imply a
compile-time constant. Case labels and subrange bounds need compile-time constants.

The primitive types are signed 64-bit `integer`, binary64 `real`, `boolean`, and
immutable Unicode `string`. There is no separate character type. Decimal integers
allow single underscores between digits; hexadecimal integers begin with `$`.
A real literal needs digits on both sides of its decimal point (`1.5e2`, not `1e2`).
Strings use single quotes and double an embedded apostrophe. Decimal `#N` literals
denote one Unicode scalar in `0..1114111`, excluding `55296..57343`, and concatenate
with adjacent string elements. Digit separators apply: `#8_364` is `€` and
`#128512` is `😀`. Invalid codes fail lexically; `string.Chr(N)` uses the same
scalar domain with runtime validation. String indexing returns one scalar as a string, not a byte,
grapheme cluster, or terminal cell. Comments begin with `//`; adjacent standalone
comment blocks also supply Markdown declaration documentation.

Declarations outside statement lists (including before a routine's `begin`),
fields, parameters and return types require type annotations.
Every fully determined statement-local initializer can omit its annotation,
including calls, factories, operators, field/index reads, closures with complete
signatures and `if`/`case` expressions. Its inferred type stays fixed. Collection
elements supply joint evidence (`[None, Some(1)]` infers Option elements).
Decision branches also supply joint evidence. Empty arrays/dicts and lone
`None`, `Ok`, `Error` need context; a single Result
constructor cannot determine both type arguments. Inference never promotes numbers.

Precedence, high to low: postfix operations (including record `with`), unary minus
and `try`, `* / div mod`, `+ -`, comparisons / `in` / nonbinding `is`, `not`,
then `and or xor`. Comparisons do not chain. Parenthesize mixtures of different
logical operators, e.g. `(A and B) or C`; repeated `A and B and C` is allowed.
`and` and `or` short-circuit left to right; `xor` evaluates both operands.
`/` performs real division; `div` and `mod` take integers. Use `=` and `<>` for
equality and inequality. Arithmetic does not implicitly convert a `real` to `integer`.

Numeric operands must have the same type; import `uses Std.Conv as Conv;` and use
`Conv.IntToReal` explicitly before mixing integer and real calculations or
comparisons. `/` on two integers still produces real. Integer `+`, `-`, `*`, and
negation reject overflow. Real operands and results must be finite; division by
either sign of zero is an error.
`div` truncates toward zero and `mod` follows the dividend's sign; the minimum
integer modulo -1 is zero. Direct minimum literals are `-9223372036854775808`
and `-$8000000000000000`; their positive magnitudes are invalid integer values.
Arithmetic permits subnormal values and signed zero; nonzero literal/text input
rounded to zero is rejected. `IntToReal` can lose integer precision. With
`uses Std.Math as Math;`, `Math.Round` uses ties to even; `Math.Trunc` truncates
toward zero, with checked integer bounds. `RealToStr` preserves the identical
binary64 parsing round-trip, including `1.0`
and `-0.0`. The Int/Real parsers in Std.Parse share literal bodies, adding an
optional sign and surrounding whitespace; real text needs a fractional part.
See [conversion](std/text/conv.md) and [parsing](std/text/parse.md).

Records, data enums, Option and Result support structural equality when all payloads
are comparable. Whole arrays, dicts, callables, tasks and channels do not support
equality. Ordering is scalar. `in` tests array values, dict keys, string substrings,
or subrange bounds. Boolean conditions never accept an integer as truthy.

```pascal id=example-1 run
program ReferenceBasics;

uses Std.Conv as Conv, Std.Math as Math, Std.Parse as Parse, Std.Test as Test;

const Limit: integer := 1000;

begin
  const Hex := 255;
  const Scale := 150.0;
  const Text := 'it''s'#10;
  var Total: integer := Hex + 1;
  begin
    const Step := 2;
    Total := Total + Step;
  end;

  Test.AssertEquals(258, Total);
  Test.AssertEquals(150.0, Scale);
  Test.AssertEquals('i', Text[0]);
  Test.AssertEquals(5, Text.Length());
  const Unicode := '€😀';
  Test.AssertEquals(2, Unicode.Length());
  Test.AssertEquals(string.Chr(8364), Unicode[0]);
  Test.AssertEquals(128512, Unicode[1].Ord());
  Test.AssertTrue((Total < Limit and Total <> 0) or false);
  Test.AssertTrue(not false xor false);
  Test.AssertEquals(2, 7 div 3);
  Test.AssertEquals(1, 7 mod 3);
  Test.AssertEquals(2.5, 5 / 2);
  const Minimum: integer := -9223372036854775808;
  Test.AssertEquals(0, Minimum mod (-1));
  Test.AssertEquals(-2, -7 div 3);
  Test.AssertEquals(1.5, Conv.IntToReal(1) + 0.5);
  Test.AssertEquals(2, Math.Round(2.5));
  Test.AssertEquals(255, Parse.Int('$FF').Unwrap());
  Test.AssertEquals('-0.0', Conv.RealToStr(Parse.Real('-0.0').Unwrap()));
  Test.AssertTrue(Parse.Real('1.0e-9999') is Error(_));
end.
```

See [basics](language/basics/README.md) and [types](language/types/README.md).

## 3. Statements, branches, loops, and expression branches

Assignments use `:=`; mutable fields and collection elements can be assignment
targets. Every statement ends with `;`, including the last one before `else`,
`elsif`, `when`, or `until`. An intentionally empty body uses `null;`.
`if` statements may omit `else`; `if` expressions require it and produce one value
per branch. Their closing `end if` gets its terminator from the surrounding expression.

`while` tests before the body; `repeat ... until Condition;` tests after it.
Counting `for` loops include both bounds, use `to` or `downto`, and declare a typed,
read-only variable. `for Item: T in Values do` evaluates Values once per loop
entry and visits array values in order or dict keys in insertion order.
The source's writable storage root is read-only throughout the body. Assignment,
InPlace, var arguments and indirect writes through helpers/callbacks are compiler
errors (FP3046); unknown callable write effects are rejected. Field/element sources
protect their containing root. Other roots, independent value copies and writes
after the loop remain legal. Var parameters that may alias a protected foreign
root are conservatively rejected for writing. Shared channel handles and closure environments
retain their resource semantics. `break;` exits
the innermost loop; `continue;` starts its next iteration. Do not change the loop
variable yourself. See [for-in](language/control-flow/for-in.md).

```pascal id=example-2 run
program ReferenceControl;

uses Std.Test as Test;

begin
  var Sum := 0;
  for I: integer := 1 to 4 do
    if I = 2 then
      continue;
    end if;

    if I = 4 then
      break;
    end if;

    Sum := Sum + I;
  end for;

  for I: integer := 2 downto 1 do
    Sum := Sum + I;
  end for;
  var Values := [1, 2];
  const Source := Values;
  for Value: integer in Source do
    Sum := Sum + Value;
    if Value = 1 then
      Values.PushInPlace(9);
    end if;
  end for;

  Test.AssertEquals(3, Values.Length());
  Test.AssertEquals(10, Sum); // Source is an independent copy; its values stay [1, 2].

  while Sum < 11 do
    Sum := Sum + 1;
  end while;

  repeat
    Sum := Sum - 1;
  until Sum = 10;
  if Sum < 0 then
    Test.AssertTrue(false);
  elsif Sum = 10 then
    null;
  else
    Test.AssertTrue(false);
  end if;
  const Sign: string := if Sum > 0 then 'positive' else 'other' end if;
  const Band: string := case Sum of
    when 0: 'zero';
    when 1..9: 'small';
    when const N if N > 9: 'large';
    else 'negative';
  end case;
  case Sum of
    when 0, 1:
      Test.AssertTrue(false);
    when 10:
      null;
    else
      Test.AssertTrue(false);
  end case;

  Test.AssertEquals('positive', Sign);
  Test.AssertEquals('large', Band);
end.
```

`case` statements and expressions use `when Pattern:`; comma-separated labels
share an arm. Scalar integer labels also allow inclusive `Low..High` ranges.
Labels and endpoints are compile-time constants; runtime tests belong in `if`
guards. A scalar binding arm (`when const N if N > 0:`) needs a guard.
Case expressions produce one expression followed by `;` per arm and must cover
every possible value. Statements use the same coverage rules. Integer/string
selectors require a fallback `else`; Boolean needs unguarded true and false,
and subranges need their entire declared interval. Boolean/subrange and closed
enum/Option/Result forbid `else`. Distinct follows its supported base type.
Guards never contribute coverage, even constant true. Duplicate labels within
an arm and labels fully covered by earlier unguarded arms are errors; partial
overlaps retain written-order matching. Use `if` for selective handling.
See [control flow](language/control-flow/README.md).

## 4. Routines, parameters, contracts, and discarded results

Declare `function Name(Parameters): ResultType;` or `procedure Name(Parameters);`.
Separate parameters with semicolons and call arguments with commas. Calls are
either entirely positional or entirely named. Fixed-signature callable values,
methods, native operations and polymorphic Std operations follow this rule;
variadic calls are entirely positional. Named calls use the visible static
signature's labels. Routine overloading is unavailable.
Functions explicitly `return Value;` on every normal path; assigning to the function
name does not return. Procedures may exit early with `return;`.

Ordinary parameters are read-only values. A `var` parameter aliases writable caller
storage and requires `var` at the call site too. Native mutating methods such as
`Items.PushInPlace(Value)` pass their receiver mutably without an explicit `var` receiver.
User record methods have read-only `Self`; they do not provide mutating receivers.

Contracts follow the routine header: zero or more `requires`, then `ensures` clauses,
then optional nested declarations and `begin`. `ensures (Returned): Expression;`
names the returned value; the colon is required. `ensures (Flag);` is an ordinary
grouped Boolean condition. Contract expressions are read-only booleans with restricted calls:
native `Length` and `IsEmpty` queries and membership through `in` are permitted;
arbitrary function calls are not. There is no
old-value snapshot, access to body locals, or mutable global state in contracts.
Contracts execute in all build modes; a failed contract is a runtime failure.
Postconditions also run on returns caused by `try` propagation.

```pascal id=example-3 run
program ReferenceRoutines;

uses Std.Test as Test;

function AddPositive(Left: integer; Right: integer): integer;
requires Left > 0;
requires Right > 0;
ensures (Answer): Answer > Left;
begin
  return Left + Right;
end function;

procedure Increment(var Value: integer);
begin
  Value := Value + 1;
  return;
end procedure;

begin
  var Count := 0;
  Increment(var Count);
  Increment(Value := var Count);
  Test.AssertEquals(2, Count);
  Test.AssertEquals(7, AddPositive(Right := 4, Left := 3));
  discard AddPositive(1, 2);
end.
```

Consume a function result in an expression or explicitly use `discard Call(...);`.
`discard` accepts every value type, including task handles, aggregates, generics,
channels, and captured callables. It does not cancel, detach, join, or consume a
task result; the owner retains the child. For side-effect work, use the
`discard go Work();` expression inside a scope. Tail calls can reuse frames when no
pending contract/scope work prevents it; do not rely on that under the debugger.
See [functions](language/functions/README.md).

## 5. Records, defaults, updates, and methods

Records are nominal value types. Construct them with the type name and named fields;
omit a field only if it has a default. `TypeName()` uses all defaults (or constructs
an empty record). Supplied expressions evaluate in written order, then omitted
defaults in field declaration order. A mutable record variable allows field
assignment; a constant record does not. Assigning a record copies its value.

`Base with Field := Value; ... end with` returns an updated copy, evaluating `Base`
once. Methods are declared inside the record. An instance method's first parameter
is `Self: RecordType` and is omitted at the call site. A `static function` or
`static procedure` has no `Self` and is called through the type. A bound method
value captures its receiver value once. Fields and methods share one
case-insensitive member namespace.

Postfix `.Field`, `[Index]`, and `.Method(...)` can follow a call result or another
primary expression. Chains evaluate left to right, evaluating receivers and arguments
once. A procedure may only finish a statement chain. Dot calls select declared record
members (including callable fields) or native operations; they never search free
functions by the type of their first parameter. Call free functions ordinarily.

```pascal id=example-4 run
program ReferenceRecords;

uses Std.Test as Test;

type Counter = record
  Value: integer := 0;

  function Add(Self: Counter; Amount: integer): Counter;
  begin
    return Self with Value := Self.Value + Amount; end with;
  end function;

  static function Zero(): Counter;
  begin
    return Counter();
  end function;
end record;

begin
  var Original: Counter := Counter.Zero();
  const Add: function(Amount: integer): Counter := Original.Add;
  Original.Value := 10;
  const Updated: Counter := Add(3);
  Test.AssertEquals(3, Updated.Value);
  Test.AssertEquals(10, Original.Value);
  Test.AssertTrue(Updated = Counter(Value := 3));
end.
```

There are no constructors with special lifecycle semantics, destructors, class
inheritance or computed properties. In units, record members need their own
`public` markers for external access; exporting the record alone does not export
every field and method. See [types](language/types/README.md).

A unit record with any private field can only be constructed inside that unit,
even if the field has a default. Export a factory to obtain values; importers can
copy them and update public fields. Exported field defaults support scalar constant
expressions and `None`; consult the record page before exposing computed defaults.

## 6. Enums, data variants, and patterns

Plain enums list semicolon-terminated members, optionally with explicit integer
backing values. Every resulting value must be unique, including automatically
assigned successors. Use qualified members (`Mode.On`) without parentheses.
Data enums add typed payload fields to variants and forbid explicit backing
values anywhere in the enum, including fieldless alternatives. Constructors
accept positional or named arguments. Payload fields use semicolons in
declarations, commas in calls.

Patterns compare literals or named compile-time constants. Bind new names explicitly
with `const Name`; `_` ignores a payload. Each payload list is all positional or
all named, every field once; labels are case-insensitive and independent of
binding names. Lists may reorder fields and nest with either form independently.
`Some`, `Ok` and `Error` use the canonical payload label `Value` in construction
and patterns. `None` and fieldless user variants have no parentheses. Guards follow the
whole pattern (`when Shape.Circle(const R) if R > 0:`). Enum, Option and Result
cases must be exhaustive, even in statement form, and forbid `else`, including
a redundant one. Guarded arms do not establish exhaustiveness. Coverage checks
descend into nested payload patterns; already covered arms are rejected.

```pascal id=example-5 run
program ReferencePatterns;

uses Std.Test as Test;

type Mode = enum
  Off;
  On = 4;
end enum;

type Shape = enum
  Dot;
  Circle(Radius: integer);
  Rect(Width: integer; Height: integer);
end enum;

function Size(Value: Shape): integer;
begin
  return case Value of
    when Shape.Dot: 0;
    when Shape.Circle(const Radius) if Radius > 0: Radius;
    when Shape.Circle(_): 0;
    when Shape.Rect(Height := const Height, Width := const Width): Width * Height;
  end case;
end function;

begin
  const Selected: Mode := Mode.On;
  Test.AssertTrue(Selected <> Mode.Off);
  Test.AssertEquals(6, Size(Shape.Rect(Width := 2, Height := 3)));
  Test.AssertEquals(4, Size(Shape.Circle(4)));
  const Nested: option of option of integer := Some(Value := Some(7));
  Test.AssertTrue(Nested is Some(Value := Some(Value := 7)));
  case Nested of
    when Some(Some(const Value)) if Value > 0:
      Test.AssertEquals(7, Value);
    when Some(_), None:
      Test.AssertTrue(false);
  end case;
  var Next: option of integer := Some(1);
  while true do
    case Next of
      when Some(const Value):
        Test.AssertEquals(1, Value);
        Next := None;
      when None:
        break;
    end case;
  end while;
end.
```

`Value is Pattern` is a nonbinding Boolean expression: it accepts grouping,
Boolean operators, guards, loop conditions, arguments, stored values, and returns.
Every `const Name` inside an `is` pattern is rejected, including nested bindings.
Use `case` arms to bind names. The selector is evaluated once; ordinary Boolean
short-circuit and logical-chain grouping rules apply. Contracts accept tests
with their usual operand restrictions.
See [pattern matching](language/pattern-matching/README.md).

## 7. Result, Option, propagation, and panic

`Result of (T, E)` has `Ok(T)` and `Error(E)`; `Option of T` has `Some(T)` and `None`.
They are values. Use nonbinding `is` for variant tests, exhaustive `case` for
payload binding, local handling, or plain defaults, and native `Unwrap` for
deliberate abort on Error/None. Case evaluates only its selected arm.
The variant tests are `Value is Ok(_)`, `Value is Error(_)`, `Value is Some(_)`,
and `Value is None`. Compute default work before case when it must run
unconditionally.
Map transforms wrapped success, AndThen composes wrapped operations, and OrElse
recovers within a wrapper. Callback argument expressions evaluate eagerly in
source order; their bodies run only on the selected variant.

Import `uses Std.Parse as Parse;`. Each parser takes `Text: string` and returns
`Result of (T, string)`:

| Call through Parse | Success type T | Input |
| --- | --- | --- |
| `Parse.Int(Text)` | integer | Signed decimal or `$` hexadecimal |
| `Parse.Real(Text)` | real | Finite real text with a fractional part |
| `Parse.Bool(Text)` | boolean | Trimmed, case-insensitive true/false |
| `Parse.HexInt(Text)` | integer | Signed hexadecimal, bare or prefixed with `$`/`0x`/`0X` |

Invalid text returns Error; `Parse.Int(Text).Unwrap()` deliberately aborts.
The parameter label is Text, including calls through matching callable values.
Std.Conv formats and converts already typed values. See [parsing](std/text/parse.md).

`try Expression` unwraps success or returns a matching failure from the enclosing
function. The operand evaluates once. Result propagation requires compatible error
types; the success type may change. Aliases are transparent, distinct errors keep
their identity, and generic errors preserve the same type parameter recursively.
There is no implicit numeric error conversion. Option propagation returns `None`
from an Option-returning function, whose success type may also differ.
This is an expression operator, not a `try` statement with a handler.
`panic(Message)` stops normal execution with a runtime failure; it does not construct
an `Error` or `None`, and there is no language catch block. `try` does not catch
panic, failed Unwrap, or other runtime failures. It is forbidden in the program
body, procedures, and functions returning an incompatible wrapper or ordinary type.

```pascal id=example-6 run
program ReferenceErrors;

uses Std.Test as Test, Std.Parse as Parse;

function Positive(Value: integer): result of (integer, string);
begin
  if Value <= 0 then
    return Error('positive value required');
  end if;

  return Ok(Value);
end function;

function Twice(Value: integer): result of (integer, string);
begin
  const Checked: integer := try Positive(Value);
  return Ok(Checked * 2);
end function;

function IsPositiveText(Text: string): result of (boolean, string);
begin
  const Value := try Parse.Int(Text := Text);
  return Ok(Value > 0);
end function;

function Next(Value: option of integer): option of integer;
begin
  return Some((try Value) + 1);
end function;

function MustBePositive(Value: integer): integer;
begin
  if Value <= 0 then
    panic('positive value required');
  end if;

  return Value;
end function;

begin
  const Good: result of (integer, string) := Twice(3);
  const Bad: result of (integer, string) := Twice(0);
  Test.AssertEquals(6, Good.Unwrap());
  case Bad of
    when Ok(_):
      Test.AssertTrue(false);
    when Error(const Message):
      Test.AssertEquals('positive value required', Message);
  end case;
  const Read: function(Text: string): result of (integer, string) := Parse.Int;
  Test.AssertTrue(Read(Text := '42') = Ok(42));
  Test.AssertTrue(Parse.Int('10') = Ok(10));
  Test.AssertTrue(Parse.HexInt('10') = Ok(16));
  Test.AssertTrue(Parse.Bool(' TRUE ') = Ok(true));
  Test.AssertTrue(IsPositiveText(Text := '3') = Ok(true));
  Test.AssertTrue(IsPositiveText('-1') = Ok(false));
  Test.AssertTrue(IsPositiveText('invalid') is Error(_));
  const Fallback: integer := case Parse.Int('invalid') of
    when Ok(const Value): Value;
    when Error(_): 0;
  end case;
  Test.AssertEquals(0, Fallback);
  const Missing: option of integer := Next(None);
  Test.AssertTrue(Missing = None);
  Test.AssertEquals(4, Next(Some(3)).Unwrap());
  Test.AssertEquals(1, MustBePositive(1));
end.
```

See [error handling](language/error-handling/README.md).

## 8. Arrays, dicts, strings, and native methods

`array of T` and `dict of K to V` are value collections with copy-on-write storage.
Changing one binding leaves previously copied collections unchanged. Array indexing
is zero-based. Array literals use `[A, B]`; dict literals use `[Key: Value]`; empty
literals are `[]` and `[:]` and need a contextual type. Dict keys require equality:
scalars and comparable aggregates are allowed; arrays, dicts, callables, tasks,
channels and aggregates containing them are not. Generic key parameters require
`Comparable`. Reading an out-of-range index or missing dict key fails at runtime;
`Lookup.Get(Key)` returns an Option for a possibly absent key.

Native operations use dot syntax: `Items.Length()`, `Items.PushInPlace(Value)`,
`Text.Split(',')`, `Parts.Join(',')`. Native methods/factories do not need `uses`.
InPlace mutators require a writable receiver and operate on that binding. Other
transformations return new values: Push returns an array; Pop returns the built-in
ArrayPopResult of T record with Remaining: array of T and Value: T. PopInPlace
returns only T. Both Pop forms fail with FP5003 on empty arrays. Old free-function
spellings such as `Push(var Items, Value)` are not native aliases. String InPlace
operations replace writable string variables; string elements remain read-only. Higher-order
methods take ordinary callable values; define a named function or an anonymous one.
Use `Text[Index]` for scalar reads and `in` for array elements, string substrings,
dictionary keys, and subrange membership. `RepeatStr`/`RepeatStrInPlace` repeat
any string; user APIs requiring one scalar declare `requires Text.Length() = 1;`.
Array `IndexOf`/`FindIndex` and string `IndexOf`/`LastIndexOf` return `Option of integer`:
`Some(Index)` or `None`. An empty substring matches first at zero and last at
`Text.Length()`. Indexes count Unicode scalars. `Sort`/`SortInPlace` are ascending
and stable, using `<`; unsupported element types fail checking even when empty.
Sorting generics require `T: Comparable` or `T: Numeric`.
`Items.SortBy(F)` and `Items.SortByInPlace(F)` sort by a named or anonymous unary
key function. Only its return type must be Comparable (generic `K: Comparable`
or `K: Numeric`); records can be elements. Both sorts are stable and ascending.
The function runs once per element in original order over a snapshot; empty
arrays make no calls. InPlace commits only after success.
String formatting uses Pascal specifiers (`'%d'.Format(7)`, `'%s'.Format('text')`),
with positional arguments; `{}` is ordinary text.

```pascal id=example-7 run
program ReferenceCollections;

uses Std.Test as Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
  var Items: array of integer := [1, 2];
  const Before: array of integer := Items;
  Items.PushInPlace(3);
  Items[0] := 5;
  const Doubled: array of integer := Items.Map(Double);
  Test.AssertEquals(2, Before.Length());
  Test.AssertEquals(1, Before[0]);
  Test.AssertEquals(10, Doubled[0]);
  Test.AssertTrue(3 in Items);
  const Repeated: array of integer := array.Fill(7, 2);
  const Empty: array of integer := [];
  const EmptyLookup: dict of string to integer := [:];
  Test.AssertEquals(7, Repeated[1]);
  Test.AssertTrue(Empty.IsEmpty() and EmptyLookup.IsEmpty());
  var Scores: dict of string to integer := ['red': 2];
  Scores['blue'] := 3;
  var Total := 0;
  for Key: string in Scores do
    Total := Total + Scores[Key];
  end for;

  Test.AssertEquals(5, Total);
  Test.AssertTrue('red' in Scores);
  Test.AssertEquals(2, Scores.Get('red').Unwrap());
  const Text := 'a,b';
  const Parts: array of string := Text.Split(',');
  Test.AssertEquals('a-b', Parts.Join('-'));
  Test.AssertTrue('a' in Text);
  Test.AssertEquals('λ', string.Chr(955));
  Test.AssertTrue(Items.IndexOf(3) = Some(2));
  Test.AssertTrue(Items.IndexOf(99) = None);
  const Ordered := Items.Sort();
  Test.AssertEquals(2, Ordered[0]);
  const Descending := Items.SortBy(
    function(Value: integer): integer begin
      return - Value;
    end function
  );
  Test.AssertEquals(5, Descending[0]);
  Test.AssertTrue('aé😀'.IndexOf('😀') = Some(2));
  Test.AssertTrue('aé😀'.LastIndexOf('') = Some(3));
  Test.AssertEquals('😀', 'aé😀'[2]);
  Test.AssertEquals('abab', 'ab'.RepeatStr(2));
end.
```

See [array](language/types/array/README.md), [dict](language/types/dictionaries.md),
and [string](language/types/string/README.md) for exact native signatures.

## 9. Callable values, anonymous routines, and captures

A callable type is `function(X: T): R` or `procedure(X: T)`. Parameter names are
required, case-insensitively unique and retained as static call labels; they do
not affect positional type/mode compatibility. A routine with parameter `N`
may implement `function(Value: integer): integer`, but the receiving value is
called with `Value := …`. Inference retains the inferred signature's labels;
compatible decision branches retain the first branch's labels. Pass a named routine
without `@`, store it in a typed binding, then call that binding. Anonymous routines
use a full `function(...)...begin...end function` or `procedure(...)...begin...end procedure`
expression. There is no expression-only arrow syntax. A returned callable must be
bound before invocation; `MakeAdder(3)(5)` is unsupported.

Nested named routines are declared before the enclosing routine's `begin`, after
its contracts. They may use enclosing parameters/locals in scope. Captured ordinary
parameters and `const` bindings are snapshots. A captured local `var` is a shared
mutable cell across closures that capture it, even after the outer call returns.
A `var` parameter cannot be captured. Globals are resolved normally rather than
captured into closure storage. A closure with mutable captures belongs to its task:
it cannot be sent through a channel, returned across tasks, or passed to `go`.

Named sibling signatures are visible throughout their lexical scope; free
routines, methods, and nested helpers follow the same rule and may be mutually
recursive. The nearest lexical declaration wins, including before its source
position. Calls and capture creation require initialized dependencies, including
through callbacks and defaults; unsafe or unprovable uses produce FP3041.

```pascal id=example-8 run
program ReferenceCallables;

uses Std.Test as Test;

function MakeAdder(Base: integer): function(Value: integer): integer;
function Add(Value: integer): integer;
begin
  return Base + Value;
end function;
begin
  return Add;
end function;

begin
  const AddThree: function(Input: integer): integer := MakeAdder(3);
  Test.AssertEquals(8, AddThree(Input := 5));
  var Count := 0;
  const Tick: procedure() := procedure() begin
    Count := Count + 1;
  end procedure;

  Tick();
  Tick();
  const Values: array of integer := [1, 2];
  const Shifted: array of integer := Values.Map(
    function(Value: integer): integer begin
      return Value + 10;
    end function
  );
  Test.AssertEquals(2, Count);
  Test.AssertEquals(12, Shifted[1]);
end.
```

See [functions](language/functions/README.md).

## 10. Generic types, routines, constraints, and recursion

Generic type declarations use `of`: `type Box of T = record ... end record;` and
`type Pair of (K, V) = ...`. Applications use `Box of integer` or
`Pair of (integer, string)`. Generic routines and record methods declare angle-bracket
parameters (`function Identity<T>(Value: T): T;`). Calls infer from supplied values
first, then fill only missing parameters from the expected result type:
`const X: Option of integer := Empty();` determines T for an `Empty<T>` factory.
Annotation, assignment, return, argument, field and callable signatures supply
context, including native callbacks; conflicts and unresolved parameters are
errors. A typed callable value
can instantiate a generic routine. Explicit angle-bracket
call arguments are unsupported. Constructors use the plain
type name with ordinary fields/arguments, and infer from arguments or the expected
type; do not write `Box of integer(Value := 1)` as a constructor expression.

Constraints are `Numeric`, `Comparable`, and `Printable`, written `T: Numeric`.
`Numeric` allows integer/real arithmetic and implies the other two; `div` and `mod`
are not valid generic Numeric operations because real is also a possible type.
Numeric operands must use the same type parameter; `+`, `-`, `*` retain it,
while `/` returns real for both integer and real instantiations.
`Comparable` permits scalar ordering and sorting, including enums without payload
fields, distinct scalars, and subranges; equality-only aggregate types do not
satisfy it. `Printable` expresses printable values.
Aliases preserve the underlying constraints; distinct types and subranges have
specific rules below. Unconstrained type parameters still permit storage/passing.
Generic field defaults must work for every instantiation: `Option of T := None`
is useful, whereas an integer default cannot stand in for arbitrary `T`.

Types are visible throughout their program or unit, including before their declaration.
Named routine signatures are also scope-wide. Values initialize in source order;
routine bodies may refer to later enclosing values, but uses must be proved safe
at the call/capture/construction point. Callable bindings initialize as values.
Recursive types must have finite values. A recursive enum needs a base variant;
recursive records can break the cycle through an Option. Generic recursive
references preserve the same type parameters. Functions can recurse and return
normally with explicit `return`.

```pascal id=example-9 run
program ReferenceGenerics;

uses Std.Console as Console, Std.Test as Test;

type Box of T = record
  Value: T;

  function Map<U>(Self: Box of T; Transform: function(Value: T): U): U;
  begin
    return Transform(Self.Value);
  end function;
end record;

type Pair of (K: Comparable, V) = record
  Key: K;
  Value: V;
end record;

type Chain of T = enum
  Empty;
  Link(Value: T; Tail: Chain of T);
end enum;

function Add<T: Numeric>(Left: T; Right: T): T;
begin
  return Left + Right;
end function;

function Divide<T: Numeric>(Left: T; Right: T): real;
begin
  return Left / Right;
end function;

function Smaller<T: Comparable>(Left: T; Right: T): T;
begin
  return if Left < Right then Left else Right end if;
end function;

procedure PrintValue<T: Printable>(Value: T);
begin
  Console.WriteLn(Value);
end procedure;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

function Missing<T>(): option of T;
begin
  return None;
end function;

function Length(Value: Chain of integer): integer;
begin
  return case Value of
    when Chain.Empty: 0;
    when Chain.Link(_, const Tail): 1 + Length(Tail);
  end case;
end function;

begin
  const Wrapped: Box of integer := Box(Value := 4);
  const Entry: Pair of (integer, string) := Pair(Key := 1, Value := 'one');
  const EndOfChain: Chain of integer := Chain.Empty;
  const Values: Chain of integer := Chain.Link(1, EndOfChain);
  const Absent: option of integer := Missing();
  Test.AssertTrue(Absent is None);
  Test.AssertEquals(4, Wrapped.Value);
  Test.AssertEquals(8, Wrapped.Map(Double));
  Test.AssertEquals('one', Entry.Value);
  Test.AssertEquals(5, Add(2, 3));
  Test.AssertEquals(4.0, Add(1.5, 2.5));
  Test.AssertEquals(0.5, Divide(1, 2));
  Test.AssertEquals(0.5, Divide(1.0, 2.0));
  Test.AssertEquals('a', Smaller('b', 'a'));
  Test.AssertEquals(1, Length(Values));
  PrintValue(Wrapped.Value);
end.
```

This example prints `4` after its assertions. See [generics](language/types/generics.md).

## 11. Aliases, distinct types, subranges, and conversions

`type Count = integer;` is a transparent alias. `type UserId = distinct integer;`
creates a separate nominal scalar type. Distinct types wrap only primitive integer,
real, boolean or string values. Use `UserId(Value)` to wrap and `integer(Id)` to
unwrap. There is no implicit conversion, arithmetic, native-method forwarding, or
printing of a distinct value; unwrap first. Same-type comparisons are supported.
Distinct types satisfy `Comparable`, but not `Numeric` or `Printable`.

`type Percent = 0..100;` declares an inclusive integer subrange with static bounds.
`Percent(Value)` checks at runtime when necessary; an integer does not implicitly
narrow into a subrange. A subrange widens to integer, and arithmetic produces ordinary
integer/real values; explicitly convert to validate the result. `Value in Percent`
tests the bounds without converting. Subranges with equal bounds are compatible;
different bounds need explicit conversion. They satisfy `Comparable` and `Printable`,
but not `Numeric`. Converting real to integer is not an `integer(...)` cast;
use the appropriate `Std.Math` rounding operation. Other conversions live in `Std.Conv`.

```pascal id=example-10 run
program ReferenceScalarTypes;

uses Std.Test as Test;

type Count = integer;

type UserId = distinct integer;

type Percent = 0..100;

begin
  const Number: Count := 7;
  const Id: UserId := UserId(Number);
  Test.AssertEquals(7, integer(Id));
  Test.AssertTrue(Id = UserId(7));
  const Half: Percent := Percent(50);
  const Next: Percent := Percent(Half + 1);
  const Wide: integer := Next;
  Test.AssertEquals(51, Wide);
  Test.AssertTrue(100 in Percent);
  Test.AssertTrue(not (101 in Percent));
end.
```

See [types](language/types/README.md).

## 12. Tasks, scopes, and channels

`go Routine(Arguments)` starts a task. As an expression it returns `task of T`
for a function returning T, or `task` for a procedure with no result. Bare
`task` has that fixed meaning in every annotation and never infers T. Every go
requires a scope in the same routine; bind its handle or write `discard go Work();`.
Standalone go is FP3022; missing ownership is FP3038. `Wait`, channels and cancellation APIs require
`uses Std.Tasks as Tasks;`. `Tasks.Wait(Handle)` consumes the retained result once; procedure
tasks use `Tasks.Wait(Handle);` without a value. WaitAll/WaitAny use homogeneous task
arrays; mixed completion selection uses TaskCase/Select with original typed handles.
Spawned result types must be known from the call; explicitly type unresolved
inputs first. Group workers require a fully instantiated callable with exactly
one by-value CancellationToken parameter.
Values passed between tasks must meet transfer rules; mutable-capture callables do not.

`scope ... end scope;` owns tasks spawned lexically inside it. Normal exit joins
children. `return`, `break`, `continue`, failed `try`, or runtime failure leaving a
scope cancels and joins outstanding children before leaving. Called routines do not
inherit the caller's lexical task ownership. Handles cannot escape their owning scope.
A child's `Error(...)` result is an ordinary value; a child panic fails the scope.
Returned Error values are ordinary data for both scope and group children.
Groups are cancelled and fully closed before their creating task publishes its
terminal outcome, including main and debugger execution. Retrieve required data
with Wait before its owner closes; discard does not retrieve it. Wait consumes
once across aliases; WaitAll, WaitAny, and TaskCase only observe, even after retrieval.
Workers cannot read or write foreign mutable variables, including globals
through helpers/callbacks. Pass snapshots, mutate worker-local copies, and use
channels. Known access is FP3042; dynamic targets retain mandatory runtime checks.
Controlled waits validate arguments and original failures first, then use
cancellation, readiness, timeout on every probe. Select instead uses ready-case
input order, including timer/cancellation cases.

Channels carry typed values. `CreateChannel(Capacity)` requires a positive capacity;
`Send` and `Receive` may block and return Results. Coordinate sends/receives and close
channels when done; `CloseChannel` reports whether it closed the channel.
Cancellation, timed operations, selection, task groups and supervision are library
APIs described in [Std.Tasks](std/concurrency/task.md), not extra language constructs.

```pascal id=example-11 run
program ReferenceTasks;

uses Std.Tasks as Tasks, Std.Test as Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

procedure Produce(Output: channel of integer);
begin
  Test.AssertTrue(Tasks.Send(Output, 9).Unwrap());
end procedure;

begin
  scope
    const Work: task of integer := go Double(21);
    const Answer: integer := Tasks.Wait(Work);
    Test.AssertEquals(42, Answer);
    const Queue: channel of integer := Tasks.CreateChannel(1);
    discard go Produce(Queue);
    const Message: integer := Tasks.Receive(Queue).Unwrap();
    Test.AssertEquals(9, Message);
    Test.AssertTrue(Tasks.CloseChannel(Queue));
  end scope;
end.
```

See [concurrency](language/concurrency/README.md).

## 13. Units, namespaces, imports, and visibility

A unit starts with `unit Qualified.Name;` and ends with `end unit;`, with optional
`uses` followed by declarations. There are no separate interface/implementation
sections or unit initialization/finalization blocks. Every declaration is private by
default. Repeat `public` on each export and separately on externally usable record
fields/methods. The `Std` namespace root is reserved for the standard library.

Save this unit as `Reference.Values.fpas` next to the following program. The tests
write the unit using its declared name so the program imports exactly this source.
Units need a project to make their sources discoverable; a standalone program-file
invocation does not automatically import neighboring files.

```pascal id=example-12 project=reference file=values.fpas
unit Reference.Values;

const Offset: integer := 1;

public type Number = record
  public Value: integer;

  public function Next(Self: Number): Number;
  begin
    return Number(Value := Self.Value + Offset);
  end function;
end record;

public function Make(Value: integer): Number;
begin
  return Number(Value := Value);
end function;
end unit;
```

Every import uses `uses UnitName as Alias;` and exposes names
only through `Alias.Name`, not unqualified or through the original unit name.
Each unit can be imported only once per source, comparing case-insensitively.
An alias cannot be shadowed by bindings, parameters or other declarations.
`as` is contextual in `uses`; it is not a general cast operator.

```pascal id=example-13 project=reference file=imports_test.fpas run
program ReferenceImports;

uses Reference.Values as Values, Std.Test as Test;

begin
  const First: Values.Number := Values.Make(2);
  const Second: Values.Number := First.Next();
  Test.AssertEquals(2, First.Value);
  Test.AssertEquals(3, Second.Value);
end.
```

Project dependencies make sources available but never replace `uses`. Multi-file
applications and libraries use `.fpasprj`; `.fpasworkspace` groups projects. Sources
and manifests are authoritative; `.fpascu` files are derived source-adjacent unit
artifacts, not editable package definitions. See [units](program-structure/units.md)
and [projects](program-structure/projects.md).

To run this pair, save the program as `ReferenceImports.fpas`. Create
`reference.fpasprj` with a `[project]` section containing `name = "reference"`,
`kind = "program"`, and `main = "ReferenceImports.fpas"`, plus a `[sources]` section
containing `include = ["Reference.Values.fpas"]`. Run `fpas run reference.fpasprj`.
The reference test generates this same manifest shape; it also checks the unit
in a library project (`kind = "library"`, the same sources, no `main`).

## 14. Find APIs and verify the source

Use native dot operations for arrays, dicts, strings, Option and Result. For hosted
services, import the owning unit with `uses Unit.Path as Alias;` and call `Alias.Name`:

| Need | Starting point |
| --- | --- |
| Console input/output | `Std.Console` (`WriteLn` needs this import) |
| Assertions and test fixtures | `Std.Test` |
| Numeric operations / bits | `Std.Math`, `Std.Bits` |
| Text parsing / typed conversion and formatting | `Std.Parse` (Result parsers), `Std.Conv` |
| Files, paths, environment, arguments, processes | `Std.Fs`, `Std.Path`, `Std.Env`, `Std.Args`, `Std.Proc` |
| Time, tasks, channels | `Std.Time`, `Std.Tasks` |
| JSON, networking, terminal UI | [Standard library index](std/README.md) |

Run `fpas doc` for exact signatures instead of guessing: `fpas doc Std.Fs.ReadText`,
`fpas doc string.TrimLeft`, or `fpas doc Std.Fs` to list a unit. It prints the
documentation and names the owning handbook page; `--report json` returns the same as
data. Do not infer a library API from Delphi or an old example using free collection
helper functions.

For source navigation, use `fpas query refs file.fpas:12:5`, `fpas query def
file.fpas:12:5`, or `fpas query type file.fpas:12:5`. Each returns one JSON document
without writing artifacts. Lines and Unicode-scalar columns start at 1;
`--project <path>` selects an explicit project/workspace and `refs
--include-declaration` adds the declaration. See [source queries](tools/source-queries.md).

For a project-wide rename, use `fpas query rename file.fpas:12:5 NewName
--dry-run` to validate and list edits as JSON, then omit `--dry-run` to apply them.
Select a containing workspace with `--project` when consumers must change too.
Invalid names, binding conflicts, source errors, and edits outside the selected
root or in the standard library are rejected. Unit and program names cannot be renamed.

Run `fpas fmt file.fpas`, then `fpas check file.fpas` and `fpas run file.fpas`.

Formatting uses two-space structural indentation and a fixed 100-column expression
target. It retains authored line groups and expands long calls automatically.
Multiline statement conditions place `then` on a separate line; wrapped `and`,
`or`, and `xor` occupy separate continuation lines. Follow the
[formatter style](tools/fmt-style.md) for complete layout rules.

Use `fpas test tests/` or the owning `.fpasprj` for tests; `*_test.fpas` programs
with assertions are discovered by the runner. `--std-lib lib` selects this checkout's
standard library when invoking the repository build directly. For machine-readable
diagnostics use `fpas check --diagnostics json file.fpas`; its `fixes` field holds exact
edits. `fpas check --fix file.fpas` applies the machine-applicable fixes for Pascal, Delphi
and Ada habits in place; run `fpas fmt` afterwards and review what remains.

Text diagnostics include a bounded source line and caret when matching text is
available. Terminal stderr receives automatic color unless `NO_COLOR` is
nonempty; redirected output stays plain. [Diagnostic rendering](tools/diagnostics.md#text-output)
describes source validation and display alignment.

For an escaping runtime error, `fpas run file.fpas --failure-dump .temp-data/failure.json`
retains a [bounded task/frame/value report](tools/failure-dump.md); choose a fresh
file path. Successful runs create no report. Check interactive programs
without launching them in unattended validation. See [CLI](program-structure/cli.md)
and [formatter style](tools/fmt-style.md).

For a targeted stop, run `fpas debug file.fpas --break file.fpas:42 --print Total`.
The default [batch debugger](tools/debugger-batch.md) returns one JSON document
with task stacks, stored values, globals and captured output. Repeat `--print`
for separate expressions; calls and constructors are forbidden. `--script`
queues test-runner line/key input; the default whole-invocation deadline is
30 seconds. Exit 2 means runtime failure; exit 3 means a bound, missed breakpoint,
failed requested inspection or worker error. Ordinary completion exposes globals
and no expired local frames. DAP/JSONL remain explicit session protocols.
