# AP20: Nested patterns and explicit bindings

Status: complete (AP20.1–AP20.3). Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Nested patterns have stable resolution and concrete coverage diagnostics
with recursive exhaustiveness and unreachable-label checks. Bindings remain
explicit regardless of surrounding names. Rejecting `else` on closed enums is
the separate, open AP03 package; current cases accept `else`.

After AP20.1 and AP20.2, patterns bind with `const Name`, ignore fields with
`_`, nest, and compare payloads with compile-time constants; coverage is
checked recursively and covered labels are rejected. Scalar guard bindings use
`when const N if Guard:`, and a bare scalar label is always a value
comparison. `Value is Pattern` tests one pattern as an `if`, `elsif`, or
`while` condition (AP20.3).

## Decisions

- `const Name` binds a field, for example `Some(const User)`; literals and
  named constants compare; `_` ignores a field. `_` never ignores a whole
  closed-enum variant.
- Patterns nest: `when Ok(Some(const User)):`.
- `Value is Pattern` tests one variant and binds its fields. It is valid as the
  condition of `if`, `elsif`, and `while`; the bindings are visible only in the
  branch or loop body. It may be followed by `and` conditions that use the
  bindings; it cannot appear under `or` or `not`. It is not a `case` and has no
  exhaustiveness check. `is` is not a free boolean value.
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
- Pattern bindings are `const` only; there is no `var` binding. Patterns stay
  positional; named fields such as `Shape.Circle(Radius := const R)` remain
  rejected (FP3026).
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

if Msg is TuiMsg.Resize(const W, const H) and W > 0 then
  Relayout(W, H);
end if;

while Queue.Next() is Some(const Job) do
  Run(Job);
end while;

const Hit: boolean := X is Some(_);   // error: 'is' only in if, elsif, while
```

## Open decisions

None. The scalar guard-binding migration and the pattern details above were
agreed before AP20.1.

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
