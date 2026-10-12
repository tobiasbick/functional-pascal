# AP40.1: Concurrency contract

Package: [AP40](README.md). Status: Decision pending; decision work package.

## Prerequisites and scope

AP26 is the baseline; record AP30.1's typed/erased/procedure handle model first.
Compare explicit ownership passing with the current lexical boundary and
explicit unowned spawn syntax. No candidate spelling is approved yet.
Decide whether worker mutable-global access is prohibited, restricted, or
supported by a specified synchronization/visibility contract.

## Deliverable and affected areas

Record the user's decisions for named/anonymous helper starts, fallible task
results, scope/group ownership, group handle discard, shared state, and task
shutdown. Separately define ready/deadline/cancellation priorities and
observation versus single result retrieval. Keep Select's case-order semantics
explicit if it remains intentionally different. Planning files only change.

## Documentation and verification

Use [scope](../../../pascal/language/concurrency/task-scopes.md),
[capture](../../../pascal/language/functions/closures.md), and
[task API](../../../pascal/std/concurrency/task.md) examples to show outcomes
before/after helper extraction, local-to-global movement, aliasing a handle,
and simultaneous readiness/cancellation. Do not claim a synchronized memory
contract merely because individual VM accesses take locks.
