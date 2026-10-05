# AP10: Typed record construction

Status: agreed direction. Effort: medium. Completion is tracked in the
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
- A record type name used as a call target always means construction.

```pascal
const P: Point := Point(X := 10, Y := 20);
const C: Config := Config(Port := 9000);   // other fields: defaults
const Q: Point := Point(10, 20);            // error: use Point(X := 10, Y := 20)

const A: Shape := Shape.Circle(2.0);
const B: Shape := Shape.Rectangle(Width := 3.0, Height := 4.0);
```

## Dependencies

- AP09 (named argument rules; AP09.2 covers variant constructors).

AP22 and AP24 depend on this package.

## Order

AP10.1 adds typed construction next to literals. AP10.2 migrates literals.
AP10.3 removes the literal form.

## Work packages

- [ ] [AP10.1: Typed construction](01-typed-construction.md)
- [ ] [AP10.2: Migrate record literals](02-migrate-record-literals.md)
- [ ] [AP10.3: Remove record literals](03-remove-record-literals.md)

## Acceptance

One canonical structural-construction rule exposes the type and field mapping
without guessing between literals and factories.

## Reference

The reference branch `codex/syntax-changes` found that construction currently
resolves supplied fields through a map and evaluates in declaration/default
order. Evaluate supplied fields in written order, then missing defaults in
declaration order.
