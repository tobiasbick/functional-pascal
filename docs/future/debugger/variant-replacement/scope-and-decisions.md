# Scope and decisions

## Implemented scope

- Evaluate explicit, fully qualified data-enum constructors such as
  `Choice.Pair(1, 2)` and fieldless constructors such as `Choice.Empty`.
- Resolve multi-segment canonical owners such as
  `Library.Unit.Choice.Empty` as one constructor target.
- Construct `Result` and `Option` values with the existing expression IR.
- Replace complete mutable roots and writable descendants atomically through
  VM, JSONL `expression.set`, DAP `setVariable`/`setExpression`, and VS Code.
- Validate every constructor argument recursively against portable debugger
  type metadata before a runtime enum value can be created.
- Expire previous variable and frame handles after a successful commit while
  preserving them after a rejected evaluation.

## Fixed decisions

- The VM debugger is the only behavior engine. Protocol and editor layers map
  requests but do not implement separate mutation semantics.
- Enum constructors require the compiler-visible `Type.Variant` name. A bare
  variant name is rejected.
- Constructor overload resolution is exact and case-insensitive. Ambiguous or
  missing targets fail explicitly.
- Constructor values are detached and bounded by the existing evaluation
  limits. Field validation uses the same portable type rules as assignment.
- A fieldless dotted designator defers constructor fallback until its complete
  pure identifier path is available. Prefixes are not invoked speculatively.
- Failed construction or validation never mutates live state and never emits a
  DAP invalidation event.

## Consciously deferred

- Switching a variant through an old payload-child handle.
- Rebinding stale variable handles after a successful variant change.
- Partial payload construction or field-by-field synthesis of a new variant.
- Assignment of functions, tasks, cells, or opaque hosted resources.

These exclusions remain governed by
[`../deferred.md`](../deferred.md); they are not silently approximated.
