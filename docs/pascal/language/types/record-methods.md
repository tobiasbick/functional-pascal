# Record methods

Records can declare functions and procedures that operate on their data, and
**static functions** that belong to the type itself.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_method`,
`function_decl`, `procedure_decl`).

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

## Static functions

A record may declare a `static function` inside its type body. Static functions
have no implicit receiver and must not declare a `Self` parameter. They are
called through the type name:

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
  WriteLn(P.Sum())  { 7 }
end.
```

Rules:

- Call through the type: `TypeName.FunctionName(Arguments)`.
- Do not call a static function through a value (`Value.Create(...)` is an error).
- Do not call an instance method through the type (`TypeName.Sum(...)` is an error).
- Static and instance members share one case-insensitive name set; duplicates are rejected.
- FPAS has no function overloading: two static functions in one record need distinct names.
- `static procedure`, static fields, and special constructors are not part of this feature.
- A public type alias whose resolved type is a record exposes the same static functions
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
