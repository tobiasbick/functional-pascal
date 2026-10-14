# AP45: Explicit snapshot iteration

Status: agreed direction; specification and regression work pending.
Effort: small. Completion is tracked in the [central plan](../README.md).

## Problem

The [for-in specification](../../../pascal/language/control-flow/for-in.md)
does not define what happens when a loop changes or replaces its source
collection. Existing lowering captures the iterable and its length once;
dictionary iteration first obtains the keys. Collection value semantics give
this snapshot behavior, but authors should not have to infer it from lowering.

## Agreed contract

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

This freezes the existing behavior; it does not replace AP37's value semantics
or callback snapshot contract. Current handbook wording is updated with the
verification work, not merely by recording this plan.

## Dependencies and work packages

AP37's collection semantics are complete. AP44 and AP46 are independent.

- [ ] [AP45.1: Specify and verify snapshot iteration](01-iteration-contract.md)

## Open decisions

None within the selected contract.

## Acceptance

The handbook and compact reference state what is captured and what remains an
ordinary body lookup. Focused execution tests establish single evaluation,
stable visit order, and mutation/reassignment behavior for arrays and dictionaries.
