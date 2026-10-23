# AP48.9: Task transfer, captures and host boundaries

Package: [AP48](README.md). Status: planned.
Prerequisite: [AP48.8](08-memory-architecture.md), including the Q50 decision.
Effort: medium with Q50 candidate A, large with candidate B. Decisions: Q46 and Q47.

## Goal

Make cross-domain behavior explicit so local execution needs no per-access sharing
protocol. Select the rules for task arguments, results, channels, callbacks, host
exports and debugger inspection before AP48.10 and AP48.11 migrate them.

## Transfer rule (Q46)

Default: keep the delivered contract. Task admission, results, channels and session
imports reject graphs containing local cells, task-bound callables or escaping
references. Ownership-neutral values are shared without copying; callable imports
require the identical executable instance. A `const` binding does not prove
transferability. Under Q50 candidate A this rule needs no graph copy.

Copying or ownership transfer is considered only if Q50 selects candidate B or a
concrete language benefit is shown. Then specify:

- a memoized graph copy that preserves sharing within one transfer and keeps
  sender and receiver mutations isolated;
- work limits, and failure or cancellation paths that never partially publish or
  leak a destination graph;
- a lock order without deadlock for opposite-direction transfers, never holding
  two heap permits across host calls.

Zero-copy ownership transfer requires proof that the sender retains no local, host,
debugger or queued reference. It is not a prototype prerequisite.

## Boundary matrix

Record accepted values, publication point, failure diagnostic and cancellation
cleanup for each row. The delivered rule is the starting point.

| Boundary | Delivered rule | Question for the new heap |
| --- | --- | --- |
| Task spawn and groups | Reject local graphs before admission | How callable environments are admitted |
| Task result and await | Reject local results before publication | Who retains a result after the producer heap retires |
| Channels and selection | Reject before mutation; unused cases close at owner termination (Q38) | Ownership of queued messages |
| Supervised retries | An abandoned attempt closes its cases; the task keeps its domain (Q38) | Which attempt roots disappear |
| Synchronous callback | Runs in the caller's domain | Where nested roots are published |
| Reusable callback session | Retains its root domain between invocations | Behavior on concurrent invocation |
| Session import/export | Identical executable required (Q41) | Which operations retain a domain |
| Debugger | Rooted live views under domain exclusion | Detached snapshot or rooted handle; mutation serialization |
| Native blocking | Published waits are collectable; opaque calls defer (Q40) | Rooting before execution ownership is released |
| Retired domain with handles | Releasing thread services maintenance (Q39) | Maintenance without per-access guards |

## Closures and cells

Within a domain, closures capturing the same variable share one mutable cell. It
survives frame return; recursion and aggregates containing callables work. Choose
between cells allocated at creation and stack-backed cells closed at scope exit.
Stack-backed cells need closing on register growth, unwinding and suspension;
prefer the easier proof unless measurements justify that machinery.

## Host handles (Q47)

Preferred: opaque handles registered with the owning heap. A handle roots a value
with its heap and code metadata; a scoped API acquires read or mutate access.
Specify whether cross-thread access blocks, queues or fails; a handle alone never
grants access. Define cloning, release, stale-handle detection, shutdown with live
handles and the last-handle race. Releasing handles requests collection even when
no task will run again, without lost requests or deadlock against callbacks.

Native code publishes roots and yields execution before a blocking wait, or defers
collection until it returns. Re-entry needs a deliberate protocol. Finalizers never
run FPAS during sweep. Explicit close and structured cleanup stay deterministic and
do not become GC-timed.

## Acceptance

Deliver the completed matrix, the selected transfer rule, diagnostics for rejected
transfers, interrupted-publication paths and the host-access lifecycle. Resolve
every path by which a domain can retire with live exports. Include examples for
shared local counters, a closure returning from its frame, task results, queued
messages, host retention and debugger access during suspension.

Likely owners: VM `tasks/`, `domain/transfer.rs`, `callback.rs`, `inspection/`,
runtime resource values, `fpas-sema` task-bound checks and the Rust embedding APIs.
Inventory exact paths before changing them; AP48.11 updates the manuals.
