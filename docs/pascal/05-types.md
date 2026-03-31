# 5. Types

## Records

Records group related data together.

### Declaring a Record

```pascal
type
  Point = record
    X: real;
    Y: real;
  end;
```

### Creating a Record

```pascal
var
  P: Point := record
    X := 0.0;
    Y := 5.0;
  end;
```

### Accessing Fields

```pascal
var
  PosX: real := P.X;
```

### Record Methods

Records can declare functions and procedures that operate on their data. The first parameter must be `Self` typed as the record. Callers use dot notation; `Self` is passed implicitly.

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

    procedure Print(Self: Point);
    begin
      WriteLn('(' + RealToStr(Self.X) + ', ' + RealToStr(Self.Y) + ')')
    end;
  end;
```

Calling methods:

```pascal
var
  A: Point := record X := 0.0; Y := 0.0; end;
  B: Point := record X := 3.0; Y := 4.0; end;
  Dist: real := A.DistanceTo(B);  { Self = A, Other = B }

begin
  A.Print()  { Self = A }
end.
```

Free-standing functions work equally well for operations on records:

```pascal
function PointToString(P: Point): string;
begin
  return '(' + RealToStr(P.X) + ', ' + RealToStr(P.Y) + ')'
end;
```

### Immutability

Record instances follow the same immutability rules as variables. A `mutable var` record allows field reassignment:

```pascal
mutable var
  P: Point := record
    X := 1.0;
    Y := 2.0;
  end;

begin
  P.X := 10.0;  { OK — P is mutable }
end.
```

### Default Field Values

A field declaration may include a default value using `:=`. When a record literal omits a field that has a default, the compiler substitutes the default automatically. Fields without a default must always be supplied.

```pascal
type
  Config = record
    Host: string := 'localhost';
    Port: integer := 8080;
    Debug: boolean := false;
  end;
```

Omitting defaulted fields:

```pascal
var
  C: Config := record end;          { Host='localhost', Port=8080, Debug=false }
  D: Config := record Port := 9000; end;  { Host='localhost', Port=9000, Debug=false }
```

Explicitly providing a value overrides the default:

```pascal
var
  E: Config := record Host := 'example.com'; Port := 443; Debug := true; end;
```

Fields without a default remain required:

```pascal
type
  Vertex = record
    Id: integer;           { required }
    X: integer := 0;       { optional }
    Y: integer := 0;       { optional }
  end;

var
  V: Vertex := record Id := 7; end;  { X=0, Y=0 from defaults }
```

### Record Update Expression

The `with` expression creates a copy of a record with selected fields replaced. The original value is never mutated.

```
base with Field := NewValue; … end
```

```pascal
type
  Point = record X: integer; Y: integer; end;

var
  P: Point := record X := 1; Y := 2; end;
  Q: Point := P with X := 99; end;   { Q.X=99, Q.Y=2; P is unchanged }
```

Multiple fields can be updated in one expression:

```pascal
var
  R: Point := P with X := 10; Y := 20; end;
```

Updates may be chained by wrapping the inner expression in parentheses:

```pascal
var
  S: Point := (P with X := 5; end) with Y := 7; end;
```

`with` works on any record value, including function return values:

```pascal
function Origin(): Point;
begin
  return record X := 0; Y := 0; end
end;

var
  T: Point := Origin() with X := 42; end;
```

Unknown field names and type mismatches in override values are compile-time errors.

## Result and Option Types

`Result of T, E` represents either a successful value of type `T` or an error value of type `E`.
`Option of T` represents either a present value of type `T` or the absence of a value.

```pascal
var Success: Result of integer, string := Ok(42);
var Failure: Result of integer, string := Error('not found');

var Present: Option of integer := Some(7);
var Missing: Option of integer := None;
```

Use `case` destructuring to handle both forms:

```pascal
case Success of
  Ok(Value): WriteLn(IntToStr(Value));
  Error(Message): WriteLn(Message)
end;

case Present of
  Some(Value): WriteLn(IntToStr(Value));
  None: WriteLn('empty')
end;
```

Use `try` to propagate `Error(...)` and `None` automatically from functions that return
`Result` or `Option`. For propagation rules, combinators, and standard-library helpers, see
[07-error-handling.md](07-error-handling.md).

## Enumerations

Enums define a set of named constants, optionally with explicit integer backing values.

### Simple Enum

```pascal
type
  Color = enum
    Red;
    Green;
    Blue;
  end;
```

Using:

```pascal
var
  C: Color := Color.Red;
```

### Enum with Backing Values

Each member can have an explicit integer value:

```pascal
type
  HttpStatus = enum
    Success = 200;
    NotFound = 404;
    InternalError = 500;
  end;
```

### Enums with Associated Data

Enum variants can carry data fields (like Rust enums or tagged unions):

```pascal
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
```

Variants with fields are constructed by calling the variant with positional arguments:

```pascal
var
  S: Shape := Shape.Circle(5.0);
  R: Shape := Shape.Rectangle(10.0, 20.0);
  P: Shape := Shape.Point;
```

Destructuring uses `case`:

```pascal
case S of
  Shape.Circle(R):
    WriteLn('Circle with radius ' + RealToStr(R));
  Shape.Rectangle(W, H):
    WriteLn('Rectangle ' + RealToStr(W) + 'x' + RealToStr(H));
  Shape.Point:
    WriteLn('Point');
end;
```

Each binding name in the pattern is positional — it corresponds to the field at that position in the variant declaration. A variant without fields (like `Point` above) uses no parentheses.

Backing values cannot be combined with associated data on the same variant.

## Arrays

Dynamic arrays that grow as needed (0-based indexing):

```pascal
var
  Numbers: array of integer := [1, 2, 3];
  Empty: array of string := [];
```

Operations:

```pascal
var
  Len: integer := Length(Numbers);  { 3 }
  First: integer := Numbers[0];              { 1 }

mutable var
  Items: array of integer := [1, 2];

begin
  Push(Items, 3);  { [1, 2, 3] }
end.
```

## Dictionaries

`dict of K to V` stores key-value pairs. Keys keep insertion order when iterated with `for-in`.

```pascal
var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Empty: dict of string to integer := [:];

var AliceAge: integer := Ages['Alice'];
```

Dictionary writes require a mutable binding:

```pascal
mutable var
  Counts: dict of string to integer := ['A': 1];

begin
  Counts['A'] := 2;
  Counts['B'] := 3
end.
```

Use `Std.Dict` for helpers such as `Length`, `ContainsKey`, `Get`, `Keys`, `Values`, and `Remove`.

## Channels

`channel of T` is a typed channel used to send values of type `T` between concurrent tasks.

```pascal
uses Std.Channel;

var Ch: channel of integer := Make();
```

Sending and receiving preserve the element type:

```pascal
Send(Ch, 42);
var Value: integer := Receive(Ch);
var MaybeValue: Option of integer := TryReceive(Ch);
```

`TryReceive` returns `Some(Value)` when data is available and `None` when no value can be taken
immediately. For `go`, `select`, buffering, closing, and task handles, see
[08-concurrency.md](08-concurrency.md).

## Type Aliases

Create semantic names for existing types:

```pascal
type
  UserId = integer;
  UserName = string;
  Callback = function(Value: integer): boolean;
```

## Generics

Types and routines can be parameterized with type parameters declared in angle brackets (`<T>`). At usage sites, concrete type arguments are supplied with the `of` keyword.

### Generic Records

```pascal
type
  Pair<A, B> = record
    First: A;
    Second: B;
  end;

var
  P: Pair of integer, string := record
    First := 42;
    Second := 'hello';
  end;
```

### Generic Enums

```pascal
type
  Maybe<T> = enum
    Just(Value: T);
    Nothing;
  end;

var
  M: Maybe of string := Maybe.Just('hi');

begin
  case M of
    Maybe.Just(V): WriteLn(V);
    Maybe.Nothing: WriteLn('nothing')
  end
end.
```

### Generic Type Aliases

```pascal
type
  Box<T> = record
    Value: T;
  end;
  IntBox = Box of integer;
```

### Generic Functions and Procedures

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

Type arguments are inferred from the call-site arguments — no explicit instantiation is needed:

```pascal
var
  X: integer := Identity(42);    { T inferred as integer }
  S: string  := Identity('hi');  { T inferred as string  }
```

### Generic Record Methods

Record methods may declare their own type parameters even when the record itself is not generic.
The type parameters belong to the method, not to the surrounding record.

```pascal
type
  Box = record
    Value: integer;

    function Map<R>(Self: Box; F: function(X: integer): R): R;
    begin
      return F(Self.Value)
    end;
  end;

function ToText(X: integer): string;
begin
  return 'value=' + IntToStr(X)
end;

var
  B: Box := record Value := 42; end;
  S: string := B.Map(ToText);   { R inferred as string }
```

Method-level type parameters may also use constraints:

```pascal
type
  Accumulator = record
    function Add<T: Numeric>(Self: Accumulator; Extra: T): T;
    begin
      return Extra
    end;
  end;
```

On generic records, method-level type parameters are added on top of the record's own type parameters:

```pascal
type
  Wrapper<T> = record
    Value: T;

    function Transform<R>(Self: Wrapper of T; F: function(X: T): R): R;
    begin
      return F(Self.Value)
    end;
  end;
```

### Implementation

Generics use type erasure. The VM operates on dynamic values, so no monomorphization is needed. Type parameters are checked at compile time and erased at runtime.

### Constraints

Type parameters can be constrained to require specific capabilities from the concrete type. Constraints are written after the parameter name, separated by a colon: `<T: Constraint>`.

```pascal
type
  Ordered<T: Comparable> = record Value: T; end;
  NumBox<T: Numeric> = record Value: T; end;
  Displayable<T: Printable> = record Value: T; end;
```

When a constrained generic type is instantiated, the compiler checks that the concrete type satisfies the constraint. Violating a constraint is a compile-time error.

#### Built-in Constraints

| Constraint | Satisfied by | Description |
|------------|-------------|-------------|
| `Comparable` | `integer`, `real`, `boolean`, `char`, `string` | Supports comparison operators: `=`, `<>`, `<`, `>`, `<=`, `>=` |
| `Numeric` | `integer`, `real` | Supports arithmetic operators: `+`, `-`, `*`, `/`, `div`, `mod` |
| `Printable` | All types except `function` and `procedure` | Can be converted to a string representation |

#### Examples

```pascal
{ Constrained record — only comparable types allowed }
type
  SortedPair<T: Comparable> = record
    First: T;
    Second: T;
  end;

var
  P: SortedPair of integer := record First := 1; Second := 2; end;  { OK }
{ var Bad: SortedPair of array of integer := ...  ← compile error }
```

```pascal
{ Mixed constrained and unconstrained parameters }
type
  Entry<K: Comparable, V> = record
    Key: K;
    Value: V;
  end;

var
  E: Entry of string, integer := record Key := 'x'; Value := 42; end;
```

```pascal
{ Constrained enum }
type
  Maybe<T: Comparable> = enum
    Just(Value: T);
    Nothing;
  end;
```
