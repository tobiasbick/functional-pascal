# 7. Error Handling

Functional Pascal provides structured error handling with `Result` and `Option` types for expected failures, and `panic` for unrecoverable errors.

## The Result Type

`Result of T, E` represents either a success (`Ok`) or a failure (`Error`):

```pascal
var R: Result of integer, string := Ok(42);
var E: Result of integer, string := Error('not found');
```

### Returning Errors

```pascal
function Divide(A: integer; B: integer): Result of integer, string;
begin
  if B = 0 then
    return Error('Division by zero')
  else
    return Ok(A div B)
end;
```

### Handling Results with Case

Use `case of` with destructuring to handle both branches:

```pascal
var R: Result of integer, string := Divide(10, 0);
case R of
  Ok(V):  WriteLn('Value: ' + IntToStr(V));
  Error(E): WriteLn('Error: ' + E);
end;
```

The binding variable (`V`, `E`) is scoped to its arm body.

## The Option Type

`Option of T` represents a value that may be absent:

```pascal
var O: Option of integer := Some(42);
var N: Option of integer := None;
```

### Using Option

```pascal
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
```

### Handling Options with Case

```pascal
var Idx: Option of integer := FindIndex([10, 20, 30], 20);
case Idx of
  Some(I): WriteLn('Found at ' + IntToStr(I));
  None:    WriteLn('Not found');
end;
```

## The Try Operator

`try` propagates errors automatically. If the expression is `Error` (for Result) or `None` (for Option), the enclosing function returns that value immediately. Otherwise, the inner value is unwrapped:

```pascal
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(IntToStr(Quotient))
end;
```

`try` also works with Option:

```pascal
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
```

## Chaining Combinators

`Std.Result` and `Std.Option` provide `Map`, `AndThen`, and `OrElse` for transforming and chaining values without manual `case` destructuring. See [`Std.Result`](std/result.md) and [`Std.Option`](std/option.md) for full API details.

```pascal
uses Std.Result, Std.Conv;

function DoubleToString(V: integer): string;
begin
  return IntToStr(V * 2)
end;

var R: Result of integer, string := Ok(21);
var M: Result of string, string := Map(R, DoubleToString);
{ M = Ok('42') }
```

```pascal
uses Std.Option, Std.Conv;

function PositiveToString(V: integer): Option of string;
begin
  if V > 0 then return Some(IntToStr(V))
  else return None
end;

var O: Option of integer := Some(5);
var M: Option of string := AndThen(O, PositiveToString);
{ M = Some('5') }
```

| Combinator | Result | Option |
|------------|--------|--------|
| `Map(V, F)` | Transform `Ok` value | Transform `Some` value |
| `AndThen(V, F)` | Chain fallible operation | Chain optional lookup |
| `OrElse(V, F)` | Recover from `Error` | Provide fallback for `None` |

## Panic

Use `panic` to abort the program when an unrecoverable error occurs:

```pascal
begin
  panic('Something went terribly wrong');
end.
```

### Guarding Assumptions

```pascal
function DivideChecked(A: integer; B: integer): integer;
begin
  if B = 0 then
    panic('Division by zero');
  return A div B
end;
```

### When to Use Panic vs Result

| Use | When |
|-----|------|
| `Result` / `Option` | Expected failure conditions (user input, file not found, search miss) |
| `panic` | Programming logic errors, broken invariants, impossible cases |

## Keywords

`Result`, `Option`, `Ok`, `Error`, `Some`, `None`, `try` are reserved keywords.
