# AP11: Individual declarations

Status: complete (Q07). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

A local edit cannot accidentally inherit the wrong declaration category, and
migration preserves exported names, scopes, and recursive types.

## Decisions

- Each declaration has its own `type`, `const`, or `var` keyword; groups that
  inherit a declaration category from a preceding entry are removed.
  Repeat `public` for each exported declaration when splitting a public group.
- The rule applies where declarations are permitted. Program and unit
  declarations support `type`, `const`, and `var`; statement lists support
  local `const` and `var`. Local type declarations are unsupported.
- Q07: all types declared in the same unit or program are visible to each
  other regardless of declaration order, including mutually recursive types.
  No explicit forward type declaration is required. Constants and variables
  remain subject to declaration order.
- Enum variants and record method signatures belong to the type definition
  and are also available before that definition's source position. Existing
  member visibility and name-ambiguity rules still apply.
- Check field defaults and record method bodies at the type definition's
  original source position. Their references to constants and variables must
  obey declaration order; collecting types must not expose later values or
  reorder initialization effects.
- [Recursive types](01-order-independent-types.md#recursive-types) must allow
  finite construction. Allow recursion through `Option`, empty containers,
  and enum alternatives that can end the recursion. Reject alias-only cycles
  and mandatory field/payload cycles with no finite construction. Preserve
  already valid recursive types.

## Open decisions

None.

## Dependencies

- AP01 (reference style).

AP16 depends on this package.

## Work packages

- [x] [AP11.1: Order-independent type declarations](01-order-independent-types.md)
- [x] [AP11.2: One keyword per declaration](02-one-keyword-per-declaration.md)

## Acceptance

A local edit cannot accidentally inherit the wrong declaration category, and
migration preserves exported names, scopes, and recursive types.
