# AP10: Typed record construction

Status: complete (AP10.1–AP10.3). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

One canonical structural-construction rule exposes the type and field mapping
without guessing between literals and factories.

## Decisions

- Records are constructed as `TypeName(Field := Value, ...)`. This replaces the
  contextually typed `record ... end` literal; one construction form remains.
- Record construction is named only. `Point(10, 20)` is an error that shows the
  named form.
- Omitted fields take their declared default; fields without a default are
  required.
- Enum variants with data are constructor calls and follow AP09: fully
  positional or fully named. Patterns stay positional.
- A record with private fields can still be constructed only inside its
  declaring unit; importers use public factory functions.
- Call targets follow normal lexical and qualified name resolution. When the
  resolved declaration is a record type or an alias of one, the call means
  construction. A nearer binding keeps its normal meaning; there is no
  fallback search for a hidden record type. Types and routines cannot share
  a name in the same scope; names are compared case-insensitively.
- A type alias may be used as a construction target, for example
  `Position(X := 1, Y := 2)` after `type Position = Point;`. It constructs the
  same record type, with the same field defaults and visibility rules.
- AP10 covers concrete record types, including imported types and their
  aliases. User-defined generic record types and constructor type-argument
  inference belong to [AP24.2](../ap24-generic-data-structures/02-generic-records.md).
- Evaluate supplied fields in written order, then missing defaults in
  declaration order.

```pascal
const P: Point := Point(X := 10, Y := 20);
const C: Config := Config(Port := 9000);   // other fields: defaults
const Q: Point := Point(10, 20);            // error: use Point(X := 10, Y := 20)

type Position = Point;
const R: Position := Position(X := 1, Y := 2); // same record type as Point

const A: Shape := Shape.Circle(2.0);
const B: Shape := Shape.Rectangle(Width := 3.0, Height := 4.0);
```

## Open decisions

None for AP10. Generic record declarations and constructor type-argument
inference are owned by AP24.2.

## Dependencies

- AP09 (named argument rules; AP09.2 covers variant constructors).

AP22 and AP24 depend on this package.

## Order

AP10.1 adds typed construction next to literals. AP10.2 migrates literals.
AP10.3 removes the literal form.

## Work packages

- [x] [AP10.1: Typed construction](01-typed-construction.md)
- [x] [AP10.2: Migrate record literals](02-migrate-record-literals.md)
- [x] [AP10.3: Remove record literals](03-remove-record-literals.md)

## Acceptance

One canonical structural-construction rule exposes the type and field mapping
without guessing between literals and factories.
