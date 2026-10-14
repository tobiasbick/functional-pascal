# AP45: Explicit snapshot iteration

Status: complete (AP45.1).
Effort: small. Completion is tracked in the [central plan](../README.md).

## Design boundary

The [for-in specification](../../../pascal/language/control-flow/for-in.md)
defines snapshot iteration even when the body changes or replaces its source
collection. Existing lowering captures the iterable and its length once;
dictionary iteration first obtains the keys. AP45 makes that behavior explicit
and protects it with execution regressions.

## Implemented contract

- Evaluate the iterable expression once on entering each execution of the loop.
- Array iteration visits the original element values in order. Mutating or
  reassigning the source binding does not change the visited sequence.
- Dictionary iteration visits the original keys in insertion order. Adding,
  removing or reinserting keys does not change that sequence.
- A dictionary lookup in the body is an ordinary lookup of the current value.
  Removing a snapshotted key can therefore make a later `Values[Key]` fail.
  The loop does not implicitly snapshot such separate lookups.
- Preserve immutable loop bindings, empty-loop behavior, lexical scope,
  `break`/`continue`, and shared-resource semantics inside copied values.
  A snapshot of values is not a deep clone of channels or closure environments.
- No live-iteration mode, new keyword, collection API or concurrency guarantee.

This freezes the existing behavior and retains AP37's value semantics and
callback snapshot contract. The handbook, collection-mutation guidance and
compact reference describe the verified contract. The loop lowering and source
grammar remain unchanged.

## Dependencies and work packages

AP37's collection semantics are complete. AP44 and AP46 are independent.

- [x] [AP45.1: Specify and verify snapshot iteration](01-iteration-contract.md)

## Open decisions

No selected AP45 decisions or implementation work remain open.

## Acceptance

The handbook and compact reference state what is captured and what remains an
ordinary body lookup. Focused execution tests establish single evaluation,
stable visit order, mutation/reassignment behavior and shared-resource semantics
for arrays and dictionaries. Independent subagent review found no blocking
issues. [AP45.1](01-iteration-contract.md) records implementation ownership,
regression coverage and independent baseline verification exceptions.
