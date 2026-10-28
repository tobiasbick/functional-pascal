# Enumerations

The code fragments use `uses Std.Text.Format as Conv, Std.Console as Console;`.

Enums define a set of named constants, optionally with explicit integer backing values.

Enum types and their variants are available throughout the declaring unit or
program, including before the declaration. Recursive payloads need a finite
terminating alternative; see [type declaration order](declaration-order.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`enum_type`, `enum_pattern`).

## Simple enum

An enum declaration ends with `end enum;` after its final terminated member.

```pascal id=example-1 fragment context=declarations
type Color = enum
  Red;
  Green;
  Blue;
end enum;
```

Using:

```pascal id=example-2 fragment context=enum-color
const C: Color := Color.Red;
```

Every user enum variant requires its type qualifier: `Color.Red`, or
`Domain.Color.Red` when accessed through an imported unit alias. This applies
to values, construction, comparisons, case labels, nested patterns, and `is`
tests. Bare `Red` and imported `Domain.Red` are rejected even when an expected
type would make them unambiguous. Adding `Status.Red` or a separate type named
`Red` leaves `Color.Red` unchanged. Explicit local type aliases may also
qualify their variants, for example `LocalColor.Red`.

`Some`, `None`, `Ok`, and `Error` remain reserved Option/Result language forms.
They keep their existing spellings and require no import or type qualifier.

Enums without payload fields satisfy `Comparable`. Ordered comparisons and
array `Sort`/`SortInPlace` use their integer backing values, including explicit
values. Every variant has a distinct backing value; sorting remains stable.
Enums with payload fields do not support ordering or satisfy `Comparable`.

## Enum with backing values

Each member can have an explicit integer value:

```pascal id=example-3 fragment context=declarations
type HttpStatus = enum
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

All resulting backing values must be unique within the enum, whether written
explicitly or assigned automatically. Nonadjacent collisions are also errors.
For example, `First = 1; Restart = 0; Second;` is invalid because `Second`
automatically receives the value 1 already used by `First`. FP3044 identifies
both variants and the conflicting value. Assign an unused explicit value or
change the preceding value that determines the implicit sequence.

Equality and case matching therefore identify the same distinct variants that
the exhaustiveness checker counts. Different enums remain nominally distinct,
even when their backing values coincide. A type alias retains the original
enum's identity and values.

```pascal id=example-4 fragment context=declarations
type Limit = enum
  Last = 9223372036854775807;
  Restart = 0;
  Next; // backing value 1
end enum;
```

## Enums with associated data

Enum variants can carry data fields (like Rust enums or tagged unions):

```pascal id=example-5 fragment context=declarations
type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
  Point;
end enum;
```

Variants with fields are constructed by calling the variant with positional arguments in
field order, or with every field named:

```pascal id=example-6 fragment context=enum-shape
const S: Shape := Shape.Circle(5.0);
const R: Shape := Shape.Rectangle(10.0, 20.0);
const N: Shape := Shape.Rectangle(Height := 20.0, Width := 10.0);
const P: Shape := Shape.Point;
```

Named construction follows the [named-argument rules](../functions/parameters.md#named-arguments):
field names match without regard to case, arguments are evaluated in written order, a call is
fully positional or fully named, and every field is passed exactly once.

Local bindings in statement position may omit the annotation, for example
`const S := Shape.Circle(5.0);` or `const P := Shape.Point;`. Generic variants
require all type arguments to be determined by payloads or fixed by a type
alias. Imported enum types and exported aliases follow the same construction
rules. See [Type inference](../basics/local-variables.md#type-inference).

Destructuring uses `case`:

```pascal id=example-7 fragment context=shape-case
case S of
  when Shape.Circle(const R):
    Console.WriteLn('Circle with radius ' + Conv.RealToStr(R));
  when Shape.Rectangle(const W, const H):
    Console.WriteLn('Rectangle ' + Conv.RealToStr(W) + 'x' + Conv.RealToStr(H));
  when Shape.Point:
    Console.WriteLn('Point');
end case;
```

Values of an enum with data compare with `=` and `<>`: they are equal when they have the same
variant and equal fields, so `Shape.Circle(5.0) = Shape.Circle(5.0)` is `true`. This requires every
field of every variant to compare; see [Operators](../basics/operators.md).

Pattern fields are either all positional or all named with `Field := Pattern`.
Labels are case-insensitive and may reorder fields; `const Binding` names are
independent of field labels. Every field is explicit, using `_` for ignored
values. A variant without fields (like `Point` above) uses no parentheses.
Each field position may also compare with a literal, compile-time constant, or enum member, or contain a nested enum, Result, or Option pattern. See [nested patterns and comparisons](../pattern-matching/syntax.md#nested-patterns-and-comparisons); use an `if` guard on the `case` arm for constraints the pattern cannot express.

Parentheses in a variant declaration must contain at least one field, and semicolons
separate fields rather than terminate the list. Consequently, `Point()` and
`Rectangle(Width: real; Height: real;)` are invalid.

If any variant has associated-data fields, no variant in that enum may have an
explicit integer backing value (FP3045). This includes fieldless alternatives:
write `Empty;`, not `Empty = 7;`, alongside `Number(Value: integer);`.
The restriction applies regardless of declaration order and also to generic
enums. Fieldless alternatives remain ordinary distinct variants.

Variant names must be ordinary identifiers. Reserved words remain reserved after a type
qualifier, so declarations such as `None` and member expressions such as `KeyKind.End` are
not valid. Choose an identifier-safe API name such as `NoCommand`, `Empty`, or `EndKey`.

## Generic enums

Generic enums declare parameters after `of`, for example `type Lookup of T =
enum ... end enum;`. Variant payloads and the expected type jointly determine
their arguments; payloadless variants with undetermined arguments need context
or an alias fixing those arguments. Applications retain those
arguments in assignments, imports, and nested patterns. See
[generic enums](generics.md#generic-enums) for construction and recursive types.

## See also

- [Pattern matching](../pattern-matching/README.md)
