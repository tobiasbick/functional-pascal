# Record methods

Records can declare functions and procedures that operate on their data, and
**static functions and procedures** that belong to the type itself.

Member signatures are available throughout the declaring unit or program,
including before the record declaration. Method bodies are checked at the
record declaration's source position, so values and free routines must precede
that declaration. See [type declaration order](declaration-order.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_method`,
`function_decl`, `procedure_decl`).

Records have no computed properties (FP2018). Computed values and state changes
are ordinary methods that callers invoke with parentheses, for example
`Camera.GetZoom()`.

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
  end record;
```

The declaring unit may call `ReadValue`; importing units cannot. `Create` and
`Current` are public. Visibility applies to
instance functions, instance procedures, static functions, and static
procedures. It is valid only in unit files. Record fields use the
same rule.

## Instance methods

The first parameter must be the reserved word `Self`, typed as the record.
`Self` is valid only in that receiver position and as the receiver expression
inside an instance method. Callers use value dot notation; `Self` is passed
implicitly.

```pascal
type Point = record
  X: real;
  Y: real;

  function DistanceTo(Self: Point; Other: Point): real;
  begin
    const DX: real := Other.X - Self.X;
    const DY: real := Other.Y - Self.Y;
    return Sqrt(DX * DX + DY * DY);
  end function;

  procedure Print(Self: Point);
  begin
    WriteLn('(' + RealToStr(Self.X) + ', ' + RealToStr(Self.Y) + ')');
  end procedure;
end record;
```

Calling instance methods:

```pascal
const A: Point := Point(X := 0.0, Y := 0.0);
const B: Point := Point(X := 3.0, Y := 4.0);
const Dist: real := A.DistanceTo(B); // Self = A, Other = B

begin
  A.Print(); // Self = A
end.
```

Returned record values can keep calling instance methods (and reading fields)
without intermediate variables:

```pascal
const Next: Point := BuildOrigin().Offset(1.0, 2.0).Normalize();
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
      return Self.Base + Value;
    end function;
  end record;

const C: Counter := Counter( Base := 10 );
const AddTen: function(Value: integer): integer := C.Add;

begin
  WriteLn(AddTen(5));  // 15 — Counter.Add(C, 5)
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
- `Self` is a read-only value parameter. A method may change a local `var`
  copy; a bound method captures the receiver value.
- Static routines are ordinary named callables (`Counter.Create`), not bound
  method values. Binding a static name through a value (`C.Create`) is an error.

## Static routines

A record may declare a `static function` or `static procedure` inside its type
body. Static routines have no implicit receiver and must not declare a `Self`
parameter. They are called through the type name:

```pascal
type Point = record
  X: integer;
  Y: integer;

  static function Create(X: integer; Y: integer): Point;
  begin
    return Point(X := X, Y := Y);
  end function;

  static function Origin(): Point;
  begin
    return Point.Create(0, 0);
  end function;

  static procedure Print(Value: Point);
  begin
    WriteLn('(' + IntToStr(Value.X) + ', ' + IntToStr(Value.Y) + ')');
  end procedure;

  function Sum(Self: Point): integer;
  begin
    return Self.X + Self.Y;
  end function;
end record;
```

```pascal
const
  P: Point := Point.Create(3, 4);
  const O: Point := Point.Origin();
begin
  Point.Print(P);
  WriteLn(P.Sum());  // 7
end.
```

Rules:

- Call through the type: `TypeName.RoutineName(Arguments)`.
- A static function returns a value; a static procedure is a statement and does
  not return a value.
- Do not call a static routine through a value (`Value.Create(...)` is an error).
- Do not call an instance method through the type (`TypeName.Sum(...)` is an error).
- Static and instance members share one case-insensitive name set with fields;
  duplicates are rejected.
- FPAS has no routine overloading: static routines in one record need distinct names.
- Static fields and special constructors are not part of this feature.
- A public type alias whose resolved type is a record exposes the same static routines
  under the alias name; the callable identity remains the resolved record type.

Construction helpers often use distinct names such as `Create` and `From…`
instead of overloads:

```pascal
TuiRect.Create(X, Y, Width, Height)
TuiRect.FromEdges(Left, Top, Right, Bottom);
```

Copying a record does not need a static function; records have value semantics:

```pascal
const Copy: Point := OtherPoint;
```

## Free-standing functions

Free-standing functions work equally well for operations on records and are
called ordinarily. They are never selected by receiver lookup. Their returned
record or built-in value can start a chain, such as `PointToString(P).Trim()`.
Declared methods continue a chain by returning `Self`, another record, or a
built-in type. See [Dot calls](../functions/fluent-calls.md).

```pascal
function PointToString(P: Point): string;
begin
  return '(' + RealToStr(P.X) + ', ' + RealToStr(P.Y) + ')';
end function;
```

Method-level type parameters are documented in [Generics](generics.md#generic-record-methods).

## See also

- [Records](records.md)
- [Generics](generics.md)
- [Optional handlers](../functions/first-class.md#optional-handlers)
- [First-class functions](../functions/first-class.md)
- [Capturing closures](../functions/closures.md)
