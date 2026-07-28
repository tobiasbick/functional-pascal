# Record methods

Records can declare functions and procedures that operate on their data, and
**static functions and procedures** that belong to the type itself.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_method`,
`function_decl`, `procedure_decl`).

## Routine visibility

Record functions and procedures declared in a unit are private by default.
`public` is written directly before each exported routine, before an optional
`static`. FPAS has no visibility sections and no explicit `private` keyword.

```pascal
type
  Counter = record
    Value: integer;

    function ReadValue(Self: Counter): integer;
    public static function Create(): Counter;
    public function Current(Self: Counter): integer;
  end;
```

The declaring unit may call `ReadValue`; importing units cannot. `Create` and
`Current` are public. Visibility applies to
instance functions, instance procedures, static functions, and static
procedures. It is valid only in unit files. Record fields, properties, and
events use the same rule.

## Instance methods

The first parameter must be `Self` typed as the record. Callers use value dot
notation; `Self` is passed implicitly.

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

Calling instance methods:

```pascal
var
  A: Point := record X := 0.0; Y := 0.0; end;
  B: Point := record X := 3.0; Y := 4.0; end;
  Dist: real := A.DistanceTo(B);  { Self = A, Other = B }

begin
  A.Print()  { Self = A }
end.
```

Returned record values can keep calling instance methods (and reading fields)
without intermediate variables:

```pascal
var Next: Point := BuildOrigin().Offset(1.0, 2.0).Normalize();
```

See [Expression postfix chaining](../functions/postfix-chaining.md).

## Bound methods as values

Reading an instance method without calling it produces a callable value with the
receiver bound. The resulting type omits the implicit `Self` parameter:

```pascal
type
  Counter = record
    Base: integer;

    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;

var C: Counter := record Base := 10; end;
var AddTen: function(Value: integer): integer := C.Add;

begin
  WriteLn(AddTen(5))  { 15 — Counter.Add(C, 5) }
end.
```

Rules:

- The designator must resolve to one instance function or procedure (not a field).
- Fields take priority over methods when both share a name.
- The receiver is evaluated once and captured by value at the binding site.
  Later assignment to the source variable does not change the bound callable.
- A procedure method yields a procedure value; a function method yields a
  function value.
- Bound methods may be stored, passed, and returned wherever that callable type
  is expected — see [First-class functions](../functions/first-class.md).
- Binding a method whose `Self` is `mutable` is rejected; use a capturing
  closure that explicitly closes over a `mutable var` instead.
- Static routines are ordinary named callables (`Counter.Create`), not bound
  method values. Binding a static name through a value (`C.Create`) is an error.

## Static routines

A record may declare a `static function` or `static procedure` inside its type
body. Static routines have no implicit receiver and must not declare a `Self`
parameter. They are called through the type name:

```pascal
type
  Point = record
    X: integer;
    Y: integer;

    static function Create(X: integer; Y: integer): Point;
    begin
      return record
        X := X;
        Y := Y;
      end
    end;

    static function Origin(): Point;
    begin
      return Point.Create(0, 0)
    end;

    static procedure Print(Value: Point);
    begin
      WriteLn('(' + IntToStr(Value.X) + ', ' + IntToStr(Value.Y) + ')')
    end;

    function Sum(Self: Point): integer;
    begin
      return Self.X + Self.Y
    end;
  end;
```

```pascal
var
  P: Point := Point.Create(3, 4);
  O: Point := Point.Origin();
begin
  Point.Print(P);
  WriteLn(P.Sum())  { 7 }
end.
```

Rules:

- Call through the type: `TypeName.RoutineName(Arguments)`.
- A static function returns a value; a static procedure is a statement and does
  not return a value.
- Do not call a static routine through a value (`Value.Create(...)` is an error).
- Do not call an instance method through the type (`TypeName.Sum(...)` is an error).
- Static and instance members share one case-insensitive name set with fields and
  properties; duplicates are rejected.
- FPAS has no routine overloading: static routines in one record need distinct names.
- Static fields and special constructors are not part of this feature.
- A public type alias whose resolved type is a record exposes the same static routines
  under the alias name; the callable identity remains the resolved record type.

Construction helpers often use distinct names such as `Create` and `From…`
instead of overloads:

```pascal
TuiRect.Create(X, Y, Width, Height)
TuiRect.FromEdges(Left, Top, Right, Bottom)
```

Copying a record does not need a static function; records have value semantics:

```pascal
var Copy: Point := OtherPoint;
```

## Free-standing functions

Free-standing functions work equally well for operations on records:

```pascal
function PointToString(P: Point): string;
begin
  return '(' + RealToStr(P.X) + ', ' + RealToStr(P.Y) + ')'
end;
```

Method-level type parameters are documented in [Generics](generics.md#generic-record-methods).

## See also

- [Records](records.md)
- [Generics](generics.md)
- [Record properties](record-properties.md)
- [Record events](record-events.md)
- [First-class functions](../functions/first-class.md)
- [Capturing closures](../functions/closures.md)
