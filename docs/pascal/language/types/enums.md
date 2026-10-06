# Enumerations

Enums define a set of named constants, optionally with explicit integer backing values.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`enum_type`, `enum_pattern`).

## Simple enum

An enum declaration ends with `end enum;` after its final terminated member.

```pascal
type
  Color = enum
    Red;
    Green;
    Blue;
  end enum;
```

Using:

```pascal
var
  C: Color := Color.Red;
```

When a program defines only one enum, unqualified variant names such as `Red` may also resolve if the short name is unique. If two enums export the same variant name (for example both define `Red`), the short name becomes ambiguous: the compiler reports an error and you must use fully qualified names such as `Color.Red` and `Status.Red`. A type with the same short name hides the variant's short name, whether declared in the same unit or imported: the variant stays reachable only as `Type.Variant`.

## Enum with backing values

Each member can have an explicit integer value:

```pascal
type
  HttpStatus = enum
    Success = 200;
    NotFound = 404;
    InternalError = 500;
  end enum;
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
  end enum;
```

## Enums with associated data

Enum variants can carry data fields (like Rust enums or tagged unions):

```pascal
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end enum;
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
  when Shape.Circle(R):
    WriteLn('Circle with radius ' + RealToStr(R));
  when Shape.Rectangle(W, H):
    WriteLn((('Rectangle ' + RealToStr(W)) + 'x') + RealToStr(H));
  when Shape.Point:
    WriteLn('Point');
end case;
```

Values of an enum with data compare with `=` and `<>`: they are equal when they have the same
variant and equal fields, so `Shape.Circle(5.0) = Shape.Circle(5.0)` is `true`. This requires every
field of every variant to compare; see [Operators](../basics/operators.md).

Each binding name in the pattern is positional — it corresponds to the field at that position in the variant declaration. A variant without fields (like `Point` above) uses no parentheses.
Each field position uses a plain identifier binding; use an `if` guard on the `case` arm for extra constraints.

Parentheses in a variant declaration must contain at least one field, and semicolons
separate fields rather than terminate the list. Consequently, `Point()` and
`Rectangle(Width: real; Height: real;)` are invalid.

A variant uses either backing values or associated data fields, not both on the same variant.

Variant names must be ordinary identifiers. Reserved words remain reserved after a type
qualifier, so declarations such as `None` and member expressions such as `KeyKind.End` are
not valid. Choose an identifier-safe API name such as `NoCommand`, `Empty`, or `EndKey`.

## See also

- [Pattern matching](../pattern-matching/README.md)
