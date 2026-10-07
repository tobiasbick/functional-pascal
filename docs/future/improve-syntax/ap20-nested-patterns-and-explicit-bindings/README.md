# AP20: Nested patterns and explicit bindings

Status: agreed direction. Effort: large. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Nested patterns have stable resolution and concrete coverage diagnostics
without a catch-all loophole for closed enums; adding a surrounding name cannot
silently alter a pattern's meaning.

Current flat enum patterns bind plain payload identifiers. Scalar value labels
require compile-time constants after AP16.1; a bare identifier in a guarded
scalar arm can also introduce a fresh binding, while a resolved constant remains
a comparison. AP16.2 preserves that distinction during migration. AP20 adds
explicit payload bindings and stable resolution for nested patterns; it must
account for the existing scalar guard-binding form during its inventory.

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

Before AP20.1, specify how its explicit binding rule applies to the existing
bare scalar guard-binding form, including shadowing and named constants.
The agreed payload-field syntax does not yet define that migration.

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

- [ ] [AP20.1: Explicit pattern bindings](01-explicit-pattern-bindings.md)
- [ ] [AP20.2: Nested patterns](02-nested-patterns.md)
- [ ] [AP20.3: Pattern test with is](03-is-pattern-test.md)

## Acceptance

Nested patterns have stable resolution and concrete coverage diagnostics
without a catch-all loophole for closed enums; adding a surrounding name cannot
silently alter a pattern's meaning.
