# AP37: Uniform mutation visibility

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md).

## Finding

[Var arguments](../../../pascal/language/functions/var-parameters.md) make
caller mutation visible, but native `Items.Push(Value)` and `Items.Pop()`
mutate an implicit writable receiver without a marker. User record Self is
always read-only. Similar native calls such as Sort and dictionary Remove
return copies. These are documented distinctions, not implementation bugs.

## Proposed direction and open decisions

Prefer one explicit write-reference mechanism for native and user operations.
Choose receiver syntax and whether user methods may mutate Self. Alternatives
are a general mutating-method model or exclusively value-producing collection
operations; assess readability and copying cost before choosing. Preserve
copy versus mutation as a meaningful distinction, not an alias to eliminate.

## Dependencies and work packages

AP06/AP17 are complete. AP37.1 precedes AP37.2. Coordinate argument conventions
with AP36.1 before selecting syntax; AP36 implementation is not automatically
a prerequisite. AP40 retains task-boundary restrictions on writable references.

- [ ] [AP37.1: Mutation model](01-mutation-model.md)
- [ ] [AP37.2: Receiver mutation](02-receiver-mutation.md)

## Acceptance

The caller can identify mutation by one documented rule across native and
user-defined operations. Writable-root, aliasing, single-evaluation, and
lifetime protections remain intact.
