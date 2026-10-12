# AP40: Explicit task lifetimes and shared state

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md).

## Findings

- A direct spawn in a [scope](../../../pascal/language/concurrency/task-scopes.md)
  is owned; moving it into a named helper does not inherit that ownership.
  A go inside an anonymous routine written in a scope is rejected instead.
- Mutable local captures cannot cross tasks, but global variables are resolved
  outside the [capture environment](../../../pascal/language/functions/closures.md)
  and shared by workers. Separate locked reads/writes do not make
  `Counter := Counter + 1` atomic. A language-level shared-state contract is missing.
- [Task APIs](../../../pascal/std/concurrency/task.md) choose different
  ready/timeout/cancellation priorities. Wait consumes a result once, while
  WaitAll/WaitAny observe completion despite freely copyable handles.
- Task-group ownership survives an unused handle, but explicit discard is
  rejected by the type-based rule. AP32 corrects misleading terminology;
  any change to permission belongs here.

## Proposed direction and open decisions

Prefer owned tasks as the normal form and an explicit indication of unowned
execution. Decide helper-boundary ownership, mutable-global access, and any
required synchronization contract. Define discard permission by ownership
only if that model can be checked reliably. Choose consistent priority rules
for controlled waits and precise terms for observation versus result retrieval.

Keep lexical scopes and dynamic task groups: their ownership, cancellation,
and supervision roles differ. Do not add an effect system or a broad linear
type system as an incidental fix.

## Dependencies and work packages

AP26 is complete. AP40.1 follows the AP30.1 handle contract; AP40.2 requires
AP30.2 for sound typed handles. AP40.3 needs AP40.1's wait decision and only
depends on AP40.2 if its selected APIs require new ownership behavior.

- [ ] [AP40.1: Concurrency contract](01-concurrency-contract.md)
- [ ] [AP40.2: Ownership and shared state](02-ownership-and-shared-state.md)
- [ ] [AP40.3: Wait contracts](03-wait-contracts.md)

## Acceptance

Routine extraction cannot silently evade the selected ownership contract.
Shared mutable state has explicit permitted behavior, and each waiting API
states its result-consumption and competing-event policy.
