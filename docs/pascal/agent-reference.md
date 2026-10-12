# Compact FPAS language reference

Read this file first when writing Functional Pascal. It covers the implemented
language in one place; follow the topic links for detailed rules and library APIs.
The [formal grammar](../specs/grammar.ebnf) describes syntax. The
[handbook](README.md) explains behavior. Planned features belong in `docs/future/`.

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
| Pascal `const N = 3` | `const N: integer := 3;` (locals may infer some types). |
| Ada `X : Integer := 1` | `var X: integer := 1;`; use `const` for an immutable binding. |
| Assign to the function name or Delphi `Result` | `return Value;`; procedures use `return;`. |
| Pascal `end;` for every block | `end function;`, `end procedure;`, `end if;`, etc.; program `end.`. |
| Omit the semicolon before Pascal `else` | Terminate each statement with `;`, including before `else`. |
| Ada `elsif` | Also `elsif`; nested `else if` requires a second `end if`. |
| Pascal `case X of 1: ...` | `case X of when 1: ... end case;`. |
| Ada `for I in 1 .. 3 loop` | `for I: integer := 1 to 3 do ... end for;`. |
| Delphi `T.Create(...)`, positional record initialization | `Point(X := 1, Y := 2)`; record fields are named. |
| Mutating `Self` or `this` | `Self` is read-only; return an updated record. |
| Ada named actuals `X => 1` | `X := 1`; an argument list is entirely named or entirely positional. |
| Pascal `var` parameter passed as `F(X)` | Declare `var X: T`, call `F(var X)` (or `F(X := var X)`). |
| Pascal routine address `@F` | Use `F` as the callable value and `F(...)` to call. |
| Delphi `TList<Integer>` | `array of integer`; generic types use `Box of integer`. |
| Pascal one-based strings / `Char` | Zero-based Unicode scalar indexing; a character is a `string`. |
| Pascal `{comment}` / `(*comment*)`, Ada `--comment` | Only `//` comments. |
| Pascal bitwise `and`, `or`, `xor` on integers | These operators take booleans; use `Std.Bits` for bits. |
| Delphi `try` / `except` | Return `Result of (T, E)` or `Option of T`; `try Expr` propagates failure. |
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
Strings use single quotes and double an embedded apostrophe. Adjacent `#0` through
`#255` character codes concatenate with string literals. For other Unicode scalar
values use `string.Chr`. String indexing returns one scalar as a string, not a byte,
grapheme cluster, or terminal cell. Comments begin with `//`; adjacent standalone
comment blocks also supply Markdown declaration documentation.

Global bindings, fields, parameters and return types require type annotations.
Local inference accepts literals, nonempty homogeneous collection literals, typed
record/enum constructors, `Some` with a known payload type, and explicit conversions.
Annotate references, ordinary calls, operators, field/index reads, closures, record
updates and `if`/`case` expressions. Empty arrays/dicts and `None`, `Ok`, `Error`
need a contextual type; one Result constructor cannot determine both type arguments.

Precedence, high to low: postfix operations (including record `with`), unary minus
and `try`, `* / div mod`, `+ -`, comparisons / `in` / condition-only `is`, `not`,
then `and or xor`. Comparisons do not chain. Parenthesize mixtures of different
logical operators, e.g. `(A and B) or C`; repeated `A and B and C` is allowed.
`and` and `or` short-circuit left to right; `xor` evaluates both operands.
`/` performs real division; `div` and `mod` take integers. Use `=` and `<>` for
equality and inequality. Arithmetic does not implicitly convert a `real` to `integer`.

Numeric operands must have the same type; use `Std.Conv.IntToReal` explicitly
before mixing integer and real calculations or comparisons. `/` on two integers
still produces real. Integer `+`, `-`, `*`, and negation reject overflow. Real
operands and results must be finite; division by either sign of zero is an error.
`div` truncates toward zero and `mod` follows the dividend's sign; the minimum
integer modulo -1 is zero. The direct minimum literal is `-9223372036854775808`.
`Round` uses ties to even, and `RealToStr` preserves a finite real's parsing
round-trip, including `1.0` and `-0.0`.

Records, data enums, Option and Result support structural equality when all payloads
are comparable. Whole arrays, dicts, callables, tasks and channels do not support
equality. Ordering is scalar. `in` tests array values, dict keys, string substrings,
or subrange bounds. Boolean conditions never accept an integer as truthy.

```pascal
program ReferenceBasics;

uses Std.Test;

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

  AssertEquals(258, Total);
  AssertEquals(150.0, Scale);
  AssertEquals('i', Text[0]);
  AssertEquals(5, Text.Length());
  AssertTrue((Total < Limit and Total <> 0) or false);
  AssertTrue(not false xor false);
  AssertEquals(2, 7 div 3);
  AssertEquals(1, 7 mod 3);
  AssertEquals(2.5, 5 / 2);
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
read-only variable. `for Item: T in Values do` iterates array values or dict keys
(dict keys follow insertion order). `break;` exits the innermost loop; `continue;`
starts its next iteration. Do not change the loop variable yourself.

```pascal
program ReferenceControl;

uses Std.Test;

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
  const Values := [1, 2];
  for Value: integer in Values do
    Sum := Sum + Value;
  end for;

  while Sum < 11 do
    Sum := Sum + 1;
  end while;

  repeat
    Sum := Sum - 1;
  until Sum = 10;
  if Sum < 0 then
    AssertTrue(false);
  elsif Sum = 10 then
    null;
  else
    AssertTrue(false);
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
      AssertTrue(false);
    when 10:
      null;
    else
      AssertTrue(false);
  end case;

  AssertEquals('positive', Sign);
  AssertEquals('large', Band);
end.
```

`case` statements and expressions use `when Pattern:`; comma-separated labels
share an arm. Scalar integer labels also allow inclusive `Low..High` ranges.
Labels and endpoints are compile-time constants; runtime tests belong in `if`
guards. A scalar binding arm (`when const N if N > 0:`) needs a guard.
Case expressions produce one expression followed by `;` per arm and must cover
every possible value. Scalar integer/string expressions need a fallback `else`;
boolean expressions need unguarded `true` and `false` arms or an `else` fallback.
Scalar statements permit `else`
and need not be exhaustive. Closed enum/Option/Result cases have stricter rules below.
See [control flow](language/control-flow/README.md).

## 4. Routines, parameters, contracts, and discarded results

Declare `function Name(Parameters): ResultType;` or `procedure Name(Parameters);`.
Separate parameters with semicolons and call arguments with commas. Calls are
either entirely positional or entirely named. Routine overloading is unavailable.
Functions explicitly `return Value;` on every normal path; assigning to the function
name does not return. Procedures may exit early with `return;`.

Ordinary parameters are read-only values. A `var` parameter aliases writable caller
storage and requires `var` at the call site too. Native mutating methods such as
`Items.Push(Value)` pass their receiver mutably without an explicit `var` receiver.
User record methods have read-only `Self`; they do not provide mutating receivers.

Contracts follow the routine header: zero or more `requires`, then `ensures` clauses,
then optional nested declarations and `begin`. `ensures (Returned)` names the
returned value. Contract expressions are read-only booleans with restricted calls:
selected native queries such as `Length`, `IsEmpty`, string/array `Contains`, and
dict `ContainsKey` are permitted; arbitrary function calls are not. There is no
old-value snapshot, access to body locals, or mutable global state in contracts.
Contracts execute in all build modes; a failed contract is a runtime failure.
Postconditions also run on returns caused by `try` propagation.

```pascal
program ReferenceRoutines;

uses Std.Test;

function AddPositive(Left: integer; Right: integer): integer;
requires Left > 0;
requires Right > 0;
ensures (Answer) Answer > Left;
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
  AssertEquals(2, Count);
  AssertEquals(7, AddPositive(Right := 4, Left := 3));
  discard AddPositive(1, 2);
end.
```

Consume a function result in an expression or explicitly use `discard Call(...);`.
`discard` cannot throw away a task or an aggregate containing tasks; callable capture
state also affects whether discarding is allowed. For a task with no handle, use the
`go` statement subject to scope rules below. Tail calls can reuse frames when no
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

```pascal
program ReferenceRecords;

uses Std.Test;

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
  AssertEquals(3, Updated.Value);
  AssertEquals(10, Original.Value);
  AssertTrue(Updated = Counter(Value := 3));
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
backing values. Use qualified members (`Mode.On`) without parentheses. Data enums
add typed payload fields to variants; constructors accept positional or named
arguments. Payload fields use semicolons in declarations, commas in calls.

Patterns compare literals or named compile-time constants. Bind new names explicitly
with `const Name`; `_` ignores a payload. Patterns may nest. Guards follow the
whole pattern (`when Shape.Circle(const R) if R > 0:`). Enum, Option and Result
cases must be exhaustive, even in statement form, and forbid `else`, including
a redundant one. Guarded arms do not establish exhaustiveness. Coverage checks
descend into nested payload patterns; already covered arms are rejected.

```pascal
program ReferencePatterns;

uses Std.Test;

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
    when Shape.Rect(const Width, const Height): Width * Height;
  end case;
end function;

begin
  const Selected: Mode := Mode.On;
  AssertTrue(Selected <> Mode.Off);
  AssertEquals(6, Size(Shape.Rect(Width := 2, Height := 3)));
  AssertEquals(4, Size(Shape.Circle(4)));
  const Nested: option of option of integer := Some(Some(7));
  if Nested is Some(Some(const Value)) and Value > 0 then
    AssertEquals(7, Value);
  else
    AssertTrue(false);
  end if;
  var Next: option of integer := Some(1);
  while Next is Some(const Value) do
    AssertEquals(1, Value);
    Next := None;
  end while;
end.
```

`is` is a pattern test only in `if`/`elsif`/`while` conditions, alone or in an `and`
chain. Its bindings are available in later conjuncts and the successful body.
It is not a general boolean expression: no `or`, `not`, parenthesized `is` clause,
assignment, `repeat` condition or case guard containing `is`.
See [pattern matching](language/pattern-matching/README.md).

## 7. Result, Option, propagation, and panic

`Result of (T, E)` has `Ok(T)` and `Error(E)`; `Option of T` has `Some(T)` and `None`.
They are values, not exceptions. Match them with `case`/`is`, or use native methods
such as `Map`, `AndThen`, `OrElse`, and `Unwrap` without an extra import.
`Unwrap` fails at runtime on an error/absent value; prefer matching when absence
is expected. Callback-based combinators can defer work until their branch is used.

`try Expression` unwraps success or returns a matching failure from the enclosing
function. Result propagation requires the same error type; the success type may
change. Option propagation returns `None` from an Option-returning function.
This is an expression operator, not a `try` statement with a handler.
`panic(Message)` stops normal execution with a runtime failure; it does not construct
an `Error` or `None`, and there is no language catch block.

```pascal
program ReferenceErrors;

uses Std.Test;

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
  AssertEquals(6, Good.Unwrap());
  case Bad of
    when Ok(_):
      AssertTrue(false);
    when Error(const Message):
      AssertEquals('positive value required', Message);
  end case;
  const Missing: option of integer := Next(None);
  AssertTrue(Missing = None);
  AssertEquals(4, Next(Some(3)).Unwrap());
  AssertEquals(1, MustBePositive(1));
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

Native operations use dot syntax: `Items.Length()`, `Items.Push(Value)`,
`Text.Split(',')`, `Parts.Join(',')`. Native methods/factories do not need `uses`.
Mutators require a writable receiver and operate on that binding. Old free-function
spellings such as `Push(var Items, Value)` are not native aliases. String operations
return new strings; string elements are not mutable character slots. Higher-order
methods take ordinary callable values; define a named function or an anonymous one.
String formatting uses Pascal specifiers (`'%d'.Format(7)`, `'%s'.Format('text')`),
with positional arguments; `{}` is ordinary text.

```pascal
program ReferenceCollections;

uses Std.Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

begin
  var Items: array of integer := [1, 2];
  const Before: array of integer := Items;
  Items.Push(3);
  Items[0] := 5;
  const Doubled: array of integer := Items.Map(Double);
  AssertEquals(2, Before.Length());
  AssertEquals(1, Before[0]);
  AssertEquals(10, Doubled[0]);
  AssertTrue(3 in Items);
  const Repeated: array of integer := array.Fill(7, 2);
  const Empty: array of integer := [];
  const EmptyLookup: dict of string to integer := [:];
  AssertEquals(7, Repeated[1]);
  AssertTrue(Empty.IsEmpty() and EmptyLookup.IsEmpty());
  var Scores: dict of string to integer := ['red': 2];
  Scores['blue'] := 3;
  var Total := 0;
  for Key: string in Scores do
    Total := Total + Scores[Key];
  end for;

  AssertEquals(5, Total);
  AssertTrue('red' in Scores);
  AssertEquals(2, Scores.Get('red').Unwrap());
  const Text := 'a,b';
  const Parts: array of string := Text.Split(',');
  AssertEquals('a-b', Parts.Join('-'));
  AssertTrue('a' in Text);
  AssertEquals('λ', string.Chr(955));
end.
```

See [array](language/types/array/README.md), [dict](language/types/dictionaries.md),
and [string](language/types/string/README.md) for exact native signatures.

## 9. Callable values, anonymous routines, and captures

A callable type is `function(X: T): R` or `procedure(X: T)`. Pass a named routine
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

```pascal
program ReferenceCallables;

uses Std.Test;

function MakeAdder(Base: integer): function(Value: integer): integer;
function Add(Value: integer): integer;
begin
  return Base + Value;
end function;
begin
  return Add;
end function;

begin
  const AddThree: function(Value: integer): integer := MakeAdder(3);
  AssertEquals(8, AddThree(5));
  var Count := 0;
  const Tick: procedure() := procedure() begin
    Count := Count + 1;
  end procedure;

  Tick();
  Tick();
  const Values: array of integer := [1, 2];
  const Shifted: array of integer := Values.Map(function(Value: integer): integer begin
    return Value + 10;
  end function);
  AssertEquals(2, Count);
  AssertEquals(12, Shifted[1]);
end.
```

See [functions](language/functions/README.md).

## 10. Generic types, routines, constraints, and recursion

Generic type declarations use `of`: `type Box of T = record ... end record;` and
`type Pair of (K, V) = ...`. Applications use `Box of integer` or
`Pair of (integer, string)`. Generic routines and record methods declare angle-bracket
parameters (`function Identity<T>(Value: T): T;`). Calls infer type arguments;
explicit angle-bracket call arguments are unsupported. Constructors use the plain
type name with ordinary fields/arguments, and infer from arguments or the expected
type; do not write `Box of integer(Value := 1)` as a constructor expression.

Constraints are `Numeric`, `Comparable`, and `Printable`, written `T: Numeric`.
`Numeric` allows integer/real arithmetic and implies the other two; `div` and `mod`
are not valid generic Numeric operations because real is also a possible type.
`Comparable` permits scalar ordering; `Printable` expresses printable values.
Aliases preserve the underlying constraints; distinct types and subranges have
specific rules below. Unconstrained type parameters still permit storage/passing.
Generic field defaults must work for every instantiation: `Option of T := None`
is useful, whereas an integer default cannot stand in for arbitrary `T`.

Types are visible throughout their program or unit, including before their declaration.
Values and free routines retain declaration-order visibility; put them before use.
Recursive types must have finite values. A recursive enum needs a base variant;
recursive records can break the cycle through an Option. Generic recursive
references preserve the same type parameters. Functions can recurse and return
normally with explicit `return`.

```pascal
program ReferenceGenerics;

uses Std.Console, Std.Test;

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

function Smaller<T: Comparable>(Left: T; Right: T): T;
begin
  return if Left < Right then Left else Right end if;
end function;

procedure PrintValue<T: Printable>(Value: T);
begin
  WriteLn(Value);
end procedure;

function Double(Value: integer): integer;
begin
  return Value * 2;
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
  AssertEquals(4, Wrapped.Value);
  AssertEquals(8, Wrapped.Map(Double));
  AssertEquals('one', Entry.Value);
  AssertEquals(5, Add(2, 3));
  AssertEquals(4.0, Add(1.5, 2.5));
  AssertEquals('a', Smaller('b', 'a'));
  AssertEquals(1, Length(Values));
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

```pascal
program ReferenceScalarTypes;

uses Std.Test;

type Count = integer;

type UserId = distinct integer;

type Percent = 0..100;

begin
  const Number: Count := 7;
  const Id: UserId := UserId(Number);
  AssertEquals(7, integer(Id));
  AssertTrue(Id = UserId(7));
  const Half: Percent := Percent(50);
  const Next: Percent := Percent(Half + 1);
  const Wide: integer := Next;
  AssertEquals(51, Wide);
  AssertTrue(100 in Percent);
  AssertTrue(not (101 in Percent));
end.
```

See [types](language/types/README.md).

## 12. Tasks, scopes, and channels

`go Routine(Arguments)` starts a task. As an expression it returns `task of T`
for a function returning T, or `task` for a procedure with no result. Bare
`task` has that fixed meaning in every annotation and never infers T. As a
statement `go` creates no handle. `Wait`, channels and cancellation APIs require
`uses Std.Tasks;`. `Wait(Handle)` consumes the retained result once; procedure
tasks use `Wait(Handle);` without a value. WaitAll/WaitAny use homogeneous task
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
Inside a scope, a Result-returning child needs a handle so its result is handled;
it cannot be started as a `go` statement with no handle. A task started by `go`
outside a scope has no lexical scope owner and no automatic join; its expression
form still produces a waitable handle. Keep and wait for that handle when completion matters.

Channels carry typed values. `CreateChannel(Capacity)` requires a positive capacity;
`Send` and `Receive` may block and return Results. Coordinate sends/receives and close
channels when done; `CloseChannel` reports whether it closed the channel.
Cancellation, timed operations, selection, task groups and supervision are library
APIs described in [Std.Tasks](std/concurrency/task.md), not extra language constructs.

```pascal
program ReferenceTasks;

uses Std.Tasks, Std.Test;

function Double(Value: integer): integer;
begin
  return Value * 2;
end function;

procedure Produce(Output: channel of integer);
begin
  AssertTrue(Send(Output, 9).Unwrap());
end procedure;

begin
  scope
    const Work: task of integer := go Double(21);
    const Answer: integer := Wait(Work);
    AssertEquals(42, Answer);
    const Queue: channel of integer := CreateChannel(1);
    go Produce(Queue);
    const Message: integer := Receive(Queue).Unwrap();
    AssertEquals(9, Message);
    AssertTrue(CloseChannel(Queue));
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

```pascal
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

Plain `uses UnitName;` makes public names available both unqualified and fully
qualified; qualify ambiguous short names. `uses UnitName as Alias;` exposes names
only through `Alias.Name`, not unqualified or through the original unit name.
Each unit can be imported only once per source, comparing case-insensitively.
An alias cannot be shadowed by bindings, parameters or other declarations.
`as` is contextual in `uses`; it is not a general cast operator.

```pascal
program ReferenceImports;

uses Reference.Values as Values, Std.Test;

begin
  const First: Values.Number := Values.Make(2);
  const Second: Values.Number := First.Next();
  AssertEquals(2, First.Value);
  AssertEquals(3, Second.Value);
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
services, import the owning unit explicitly, even for fully qualified calls:

| Need | Starting point |
| --- | --- |
| Console input/output | `Std.Console` (`WriteLn` needs this import) |
| Assertions and test fixtures | `Std.Test` |
| Numeric operations / conversions / bits | `Std.Math`, `Std.Conv`, `Std.Bits` |
| Files, paths, environment, arguments, processes | `Std.Fs`, `Std.Path`, `Std.Env`, `Std.Args`, `Std.Proc` |
| Time, tasks, channels | `Std.Time`, `Std.Tasks` |
| JSON, parsing, networking, terminal UI | [Standard library index](std/README.md) |

Read the owning unit's handbook page or generated declarations in `lib/api/` for
exact signatures. Native methods are described under their type pages. Do not infer
a library API from Delphi or an old example using free collection helper functions.

Run `fpas fmt file.fpas`, then `fpas check file.fpas` and `fpas run file.fpas`.
Use `fpas test tests/` or the owning `.fpasprj` for tests; `*_test.fpas` programs
with assertions are discovered by the runner. `--std-lib lib` selects this checkout's
standard library when invoking the repository build directly. For machine-readable
diagnostics use `fpas check --diagnostics json file.fpas`. Check interactive programs
without launching them in unattended validation. See [CLI](program-structure/cli.md)
and [formatter style](tools/fmt-style.md).
