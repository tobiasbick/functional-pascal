# 4. Functions

Functions are the primary building block in Functional Pascal. They can be stored in variables, passed as arguments, and nested inside other functions.

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
  WriteLn(R);
end.
```

## Nested Functions

Functions can be declared inside other functions. They have access to the enclosing function's variables (lexical scope):

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

## Anonymous Functions (Lambdas)

Anonymous functions use the `function` keyword inline and can be assigned to variables or passed as arguments:

```pascal
var Square: function(X: integer): integer :=
  function(X: integer): integer
  begin
    return X * X
  end;

WriteLn(Square(4));  { 16 }
```

## Closures

Anonymous functions capture variables from their enclosing scope by value:

```pascal
function MakeAdder(N: integer): function(X: integer): integer;
begin
  return function(X: integer): integer
  begin
    return X + N
  end
end;

begin
  var Add5 := MakeAdder(5);
  WriteLn(Add5(10));  { 15 }
end.
```

## Forward Declarations

Use `forward` to declare a function before its body, enabling mutual recursion:

```pascal
function IsEven(N: integer): boolean; forward;

function IsOdd(N: integer): boolean;
begin
  if N = 0 then
    return false
  else
    return IsEven(N - 1);
end;

function IsEven(N: integer): boolean;
begin
  if N = 0 then
    return true
  else
    return IsOdd(N - 1);
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

See [Types — Generics](05-types.md#generics) for generic records, enums, and type aliases.

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
