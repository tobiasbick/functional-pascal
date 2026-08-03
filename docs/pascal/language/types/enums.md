# Enumerations

Enums define a set of named constants, optionally with explicit integer backing values.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`enum_type`, `enum_pattern`).

## Simple enum

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

When a program defines only one enum, unqualified variant names such as `Red` may also resolve if the short name is unique. If two enums export the same variant name (for example both define `Red`), the short name becomes ambiguous: the compiler reports an error and you must use fully qualified names such as `Color.Red` and `Status.Red`.

## Enum with backing values

Each member can have an explicit integer value:

```pascal
type
  HttpStatus = enum
    Success = 200;
    NotFound = 404;
    InternalError = 500;
  end;
```

Members without an explicit value start at `0` and continue with the previous member's value plus
one. An explicit value restarts that sequence. The signed 64-bit maximum
`9223372036854775807` is valid as an explicit final value, but it has no implicit successor. If a
later member needs an implicit value, the compiler reports an error; assign that member an explicit
value to restart the sequence.

```pascal
type
  Limit = enum
    Last = 9223372036854775807;
    Restart = 0;
    Next; // backing value 1
  end;
```

## Enums with associated data

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
Each field position uses a plain identifier binding; use an `if` guard on the `case` arm for extra constraints.

A variant uses either backing values or associated data fields, not both on the same variant.

Variant names must be ordinary identifiers. Reserved words remain reserved after a type
qualifier, so declarations such as `None` and member expressions such as `KeyKind.End` are
not valid. Choose an identifier-safe API name such as `NoCommand`, `Empty`, or `EndKey`.

## See also

- [Pattern matching](../pattern-matching/README.md)
