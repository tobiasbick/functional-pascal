# AP20: Nested patterns and explicit bindings

Status: complete (AP20.1–AP20.3). Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Nested patterns have stable resolution and concrete coverage diagnostics
with recursive exhaustiveness and unreachable-label checks. Bindings remain
explicit regardless of surrounding names. The separate, completed
[AP03](../ap03-explicit-closed-enum-cases/README.md) package rejects `else` on closed enums;
scalar cases retain it.

After AP20.1 and AP20.2, patterns bind with `const Name`, ignore fields with
`_`, nest, and compare payloads with compile-time constants; coverage is
checked recursively and covered labels are rejected. Scalar guard bindings use
`when const N if Guard:`, and a bare scalar label is always a value
comparison. `Value is Pattern` is a nonbinding Boolean expression; AP35.2
replaces AP20.3's original binding and condition-only rules.

## Decisions

- `const Name` binds a field, for example `Some(const User)`; literals and
  named constants compare; `_` ignores a field. `_` never ignores a whole
  closed-enum variant.
- Patterns nest: `when Ok(Some(const User)):`.
- `Value is Pattern` is a nonbinding Boolean expression with ordinary grouping
  and logical operators. It accepts nested patterns, wildcards, and compile-time
  comparisons, and has no exhaustiveness check. Every `const Name` inside it
  is rejected; only case arms introduce pattern names (AP35.2).
- `is` becomes a reserved keyword.
- Scalar guard bindings use the same explicit form:
  `when const N if N > 0:`. A bare identifier in a scalar label is always a
  value comparison and must name a compile-time constant or enum member; an
  unknown name reports a diagnostic showing `const N`. The existing limits
  stay: exactly one label in the arm and a required guard, so
  `when const N:` without a guard is rejected in favour of `else`.
- A bare identifier in a payload position (AP20.2), for example
  `Some(MaxValue)`, is a comparison. Like scalar labels it must be a
  compile-time constant; computed `const` values belong in the guard. A name
  that is not a constant reports a diagnostic showing `const Name`.
- `_` ignores exactly one field. When that field has an enum type,
  `Some(_)` covers all of its variants. Only `_` in place of a whole variant
  (`when _:`) is rejected.
- Pattern bindings are `const` only; there is no `var` binding. AP36 adds
  fully named field lists alongside positional ones, for example
  `Shape.Circle(Radius := const R)`. Every field occurs exactly once; mixing
  is rejected independently at each nesting level. Some/Ok/Error use `Value`.
- One arm may list several labels with the same binding names and compatible
  types, for example `when Ok(const Msg), Error(const Msg):`.
- Exhaustiveness is computed recursively for nested patterns: `Ok(Some(_))`,
  `Ok(None)`, and `Error(_)` together are exhaustive. Guards do not count
  toward coverage. Rejecting `else` on closed enums belongs to AP03.

```pascal
case Response of
  when Ok(Some(const User)):
    Greet(User);
  when Ok(None):
    null;
  when Error(_):
    Retry();
end case;

const Hit: boolean := X is Some(_);
if not Hit then ReportMissing(); end if;
```

## Open decisions

None. The scalar guard-binding migration and the pattern details above were
agreed before AP20.1.

## Independent follow-up

[AP35.2](../ap35-condition-composition/02-pattern-conditions.md) replaces
AP20.3's original context and binding rules and owns the consumer migration.
The rules above describe the resulting language. AP20 remains complete.

[AP36](../ap36-call-and-construction-conventions/README.md#recorded-construction-and-pattern-decisions)
implements fully named field lists alongside positional enum/Option/Result patterns,
with every field explicit and mixing rejected per nesting level. Some/Ok/Error
use Value as their named payload label. AP36.2 replaces only the positional-only
field restriction, preserving comparisons, immutable case bindings, coverage,
and AP35's nonbinding is. The rules above include this delivered extension.

## Dependencies

- AP13 (`when` arms, `elsif`, statement-list bodies).
- AP07.3 (comparison-level precedence for the AP20.3 `is` test).
- AP16.1 (constant classification for existing scalar case labels and guards).

AP03, AP15, and AP24 depend on this package.

## Order

AP20.1 introduces explicit bindings in flat patterns and migrates existing
bindings. AP20.2 adds nesting and comparisons, which need the explicit form to
stay unambiguous. AP20.3 adds the `is` test using the same pattern rules.

## Work packages

- [x] [AP20.1: Explicit pattern bindings](01-explicit-pattern-bindings.md)
- [x] [AP20.2: Nested patterns](02-nested-patterns.md)
- [x] [AP20.3: Pattern test with is](03-is-pattern-test.md)

## Acceptance

Nested patterns have stable resolution and concrete coverage diagnostics
with recursive exhaustiveness and unreachable-label checks. Named compile-time
constants contribute their values to coverage, and qualified constructors must
resolve to a variant of the matched enum. Pattern bindings cannot hide names
used by comparisons in the same label or another label of the arm, including
inside closures. The independent closed-enum `else` restriction remains in AP03.
