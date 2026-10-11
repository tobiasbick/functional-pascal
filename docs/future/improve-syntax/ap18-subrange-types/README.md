# AP18: Subrange types

Status: complete (AP18.1, AP18.2; Q10, Q11 agreed).
Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Unchecked input cannot bypass a range restriction; invalid constants produce
compile-time diagnostics and invalid dynamic conversions panic with the value
and range; widening and integer arithmetic produce integers, while `/` produces
`real`. Membership tests return `boolean` for the inclusive bounds before
explicit conversion.

## Decisions

### Base types (Q10)

- Initially support integer subranges only, such as `type Percent = 0..100;`.
- The base type follows from the bounds; no explicit base type may be written.
- Enum subranges are deferred until a concrete use case justifies them.

### Conversions and violations (Q11)

- A subrange value widens implicitly to `integer`. Integer arithmetic and unary
  negation produce `integer`; `/` retains the existing real-division rule and
  produces `real`.
- Conversion from `integer` to a subrange requires an explicit checked
  conversion, such as `Percent(X)`, including in assignments and returns.
- An out-of-range constant is a compile-time error. An out-of-range dynamic
  value causes a runtime panic that reports the value and the allowed range.
- Membership testing with `X in Percent` checks the range before conversion.
  Conversion does not return `Option` or `Result`.
- Subranges restrict ranges; they are distinct from AP19's type identity.
- No arbitrary proof solving or automatic validation of external data.

### Type compatibility

- Bounds are inclusive compile-time integer expressions. Equal endpoints define
  a single-value range; inverted bounds are invalid.
- Subranges with the same evaluated bounds are the same type, regardless of
  declaration name or owning unit. Aliases preserve that type.
- Moving a value between different subranges requires an explicit checked
  conversion, even when one range contains the other.
- Widening applies to scalar value positions, not to whole containers or callable
  signatures. `var` parameters retain exact type compatibility.

### Operators, generics, and cases

- Arithmetic and comparisons use the integer values; logical operators retain
  their Boolean-only rules. Containers preserve their declared element types.
- Subranges satisfy `Comparable` and `Printable`, but not `Numeric`: arithmetic
  cannot promise a result of the original subrange type.
- Subrange case selectors follow the existing integer case rules. Statements
  may omit `else`; expressions require it. Finite bounds do not introduce a new
  coverage analysis. Labels and endpoints remain compile-time constants.

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
- AP22.1 (local inference for explicit conversions).

## Order

AP18.1 delivers declarations and conversions together, because a subrange
without checked conversion has no way to obtain dynamic values. AP18.2 adds
membership tests.

## Work packages

- [x] [AP18.1: Subrange declarations and conversions](01-subrange-declarations.md)
- [x] [AP18.2: Subrange membership](02-subrange-membership.md)

## Acceptance

Unchecked input cannot bypass the range restriction; invalid constants produce
compile-time diagnostics and invalid dynamic conversions panic with the value
and range; widening and integer arithmetic produce integers, while `/` produces
`real`. Membership evaluates its integer or subrange input once, returns `false`
outside the bounds, and supports guarded explicit conversion with the existing
Boolean short-circuit rules.
