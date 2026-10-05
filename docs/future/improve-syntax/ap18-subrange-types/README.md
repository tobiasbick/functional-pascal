# AP18: Subrange types

Status: agreed direction (Q10, Q11). Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Unchecked input cannot bypass a range restriction; invalid constants produce
compile-time diagnostics and invalid dynamic conversions panic with the value
and range; widening and arithmetic produce integers.

## Decisions

### Base types (Q10)

- Initially support integer subranges only, such as `type Percent = 0..100;`.
- The base type follows from the bounds; no explicit base type may be written.
- Enum subranges are deferred until a concrete use case justifies them.

### Conversions and violations (Q11)

- A subrange value widens implicitly to `integer`; arithmetic results are
  `integer`.
- Conversion from `integer` to a subrange requires an explicit checked
  conversion, such as `Percent(X)`, including in assignments and returns.
- An out-of-range constant is a compile-time error. An out-of-range dynamic
  value causes a runtime panic that reports the value and the allowed range.
- Membership testing with `X in Percent` checks the range before conversion.
  Conversion does not return `Option` or `Result`.
- Subranges restrict ranges; they are distinct from AP19's type identity.
- No arbitrary proof solving or automatic validation of external data.

```pascal
if Input in Percent then
  Show(Percent(Input));
else
  Reject(Input);
end if;
```

## Dependencies

- AP07 (precedence of `in` and boolean rules).
- AP16 (compile-time constant classification for bounds, AP16.1).

## Order

AP18.1 delivers declarations and conversions together, because a subrange
without checked conversion has no way to obtain dynamic values. AP18.2 adds
membership tests.

## Work packages

- [ ] [AP18.1: Subrange declarations and conversions](01-subrange-declarations.md)
- [ ] [AP18.2: Subrange membership](02-subrange-membership.md)

## Acceptance

Unchecked input cannot bypass the range restriction; invalid constants produce
compile-time diagnostics and invalid dynamic conversions panic with the value
and range; widening and arithmetic produce integers.
