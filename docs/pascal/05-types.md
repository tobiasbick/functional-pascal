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

## Reference Types

`ref T` denotes a shared reference to a heap-allocated record of type `T`. Use it when multiple variables should refer to the same record, or when a record must contain references to values of its own type.

### Recursive Records

```pascal
type
  Node = record
    Value: integer;
    Next: Option of ref Node;
  end;
```

### Allocating with `new`

`new T with ... end` allocates a record and returns a value of type `ref T`.

```pascal
mutable var
  Head: ref Node := new Node with
    Value := 1;
    Next := None;
  end;
```

The target of `new` must be a record type. Each declared field must be initialized exactly once. Missing fields and unknown fields are compile-time errors.

### Sharing and Implicit Dereference

Assigning a `ref` value shares the same underlying record:

```pascal
mutable var
  Head: ref Node := new Node with
    Value := 1;
    Next := None;
  end;

var
  Alias: ref Node := Head;

begin
  Head.Value := 2;
  WriteLn(IntToStr(Alias.Value));  { 2 }
end.
```

Field access, field assignment, indexing, and method calls dereference `ref` values automatically. Use the same dot and bracket syntax as for ordinary records, arrays, and dicts.

### Mutability Through `ref`

The mutability rule is checked at the variable used for the write. Rebinding a `ref` variable still requires `mutable var`, and writing through a `ref` also requires a mutable binding:

```pascal
var
  Root: ref Node := new Node with
    Value := 1;
    Next := None;
  end;

begin
  Root.Value := 2;  { ERROR: Root is immutable }
end.
```

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
