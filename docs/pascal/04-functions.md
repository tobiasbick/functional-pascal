# 4. Functions

Functions are the primary building block in Functional Pascal. They can be stored in variables, passed as arguments, and nested inside other functions.

Formal syntax: [`docs/specs/grammar.ebnf`](../specs/grammar.ebnf) (`function_decl`, `procedure_decl`, `function_type`, `procedure_type`).

## Declaration shape

```text
function Name [<T>] ( [ params ] ) : RetType ;
  { nested function | nested procedure }
begin
  ...
end;

procedure Name [<T>] ( [ params ] ) ;
  { nested function | nested procedure }
begin
  ...
end;
```

- The header ends with `;` before the body. The body ends with `end;` (including top-level declarations in a program or unit).
- Use `()` when there are no parameters: `function Pi(): real;`.
- Parameter lists use `;` between parameters; call sites use `,`.

## Functions

A function returns a value using `return`:

```pascal
function Add(A: integer; B: integer): integer;
begin
  return A + B;
end;
```

## Procedures

A procedure performs an action but returns no value:

```pascal
procedure SayHello(Name: string);
begin
  WriteLn('Hello, ' + Name + '!');
end;
```

Procedures use bare `return` to exit early without a value:

```pascal
procedure LogIfPositive(mutable Count: integer; Value: integer);
begin
  if Value <= 0 then
    return;
  Count := Count + 1;
  WriteLn('logged ', Value);
end;
```

## Parameters

Parameters are separated by semicolons in declarations. Calls use commas. Each parameter requires a type annotation:

```pascal
function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value;
end;

begin
  var R: integer := Clamp(150, 0, 100);  { 100 }
end.
```

## Mutable Parameters

By default, parameters are immutable inside the routine body. Prefix a parameter with `mutable` to allow reassignment to the local binding:

```pascal
procedure Inc(mutable X: integer);
begin
  X := X + 1;
end;
```

`mutable` only affects the local binding — the caller's value is not changed. To observe changes in the caller, pass a reference type (array, record instance) and mutate its contents. See [Records — immutability](language/types/records.md#immutability) for details.

## Function Types

Function types describe the signature of a callable:

```pascal
type
  IntBinaryOp = function(A: integer; B: integer): integer;
  StringAction = procedure(S: string);
```

## First-Class Functions

Functions can be assigned to variables and passed as arguments:

```pascal
function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value);
end;

function Double(X: integer): integer;
begin
  return X * 2;
end;

begin
  var R: integer := Apply(Double, 5);  { 10 }
  var Op: function(X: integer): integer := Double;
  WriteLn(Op(7));                        { 14 }
end.
```

Call sites pass a **named** function or procedure, or a **variable** whose type is a function or procedure type. Qualified routines work the same way: `Std.Console.WriteLn(...)`.

## Nested Functions

Functions can be declared inside other functions. Use nested declarations for local helpers and mutual recursion:

```pascal
function Hypotenuse(A: real; B: real): real;

  function Square(X: real): real;
  begin
    return X * X;
  end;

begin
  return Sqrt(Square(A) + Square(B));
end;
```

## Mutual recursion

Declare callees before callers when only one direction of call is needed. For mutual recursion, nest the helper in the outer routine so both names are in scope when bodies are checked:

```pascal
function IsEven(N: integer): boolean;
  function IsOdd(X: integer): boolean;
  begin
    if X = 0 then
      return false
    else
      return IsEven(X - 1)
  end;
begin
  if N = 0 then
    return true
  else
    return IsOdd(N - 1)
end;
```

## Generic Functions

Functions and procedures can declare type parameters in angle brackets after the name:

```pascal
function Identity<T>(Value: T): T;
begin
  return Value
end;

procedure PrintValue<T>(Value: T);
begin
  WriteLn(Value)
end;
```

Type arguments are inferred from the call-site arguments:

```pascal
begin
  WriteLn(Identity(42));      { T = integer }
  WriteLn(Identity('hello')); { T = string  }
  PrintValue(3.14)            { T = real    }
end.
```

Multiple type parameters are separated by commas:

```pascal
function First<A, B>(X: A; Y: B): A;
begin
  return X
end;
```

See [Generics](language/types/generics.md) for constraints and method-level generics on record methods.

## Early Return

`return` both sets the return value and exits the function immediately:

```pascal
function IndexOf(Items: array of string; Target: string): integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
  begin
    if Items[I] = Target then
      return I;
  end;
  return -1;
end;
```
