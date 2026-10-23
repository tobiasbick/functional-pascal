# AP48.9: Task transfer, captures and host boundaries

Package: [AP48](README.md). Status: planned; boundary choices open.
Prerequisite: [AP48.8](08-memory-architecture.md) architecture decisions.
Effort: large. Decisions: Q46 and Q47.

## Goal

Make cross-domain behavior explicit enough that local execution needs no
per-access sharing protocol. Select semantics for task arguments, results,
channels, callbacks, host exports and debugger inspection before migrating them.
The user permits semantic and API changes. Syntax changes may also be proposed
without treating existing syntax as an architectural constraint, but each requires
discussion and explicit user approval before implementation. This is design
freedom, not an instruction to change behavior without documenting the replacement.

## Current contract and proposed starting point

Mutable-capture callables and graphs containing local cells are currently rejected
at task boundaries. Eligible values can be shared; callable imports also check
executable provenance. A `const` binding alone does not prove transferability.

Start with the same prohibition on foreign local references. Keeping local
mutable captures within one domain already permits a much simpler heap. Do not
assume that accepting arbitrary mutable closures across tasks is required.
Explore copying such graphs only if it has a clear language benefit and a
manageable alias contract. Changes to current transfer rules must be concrete,
testable decisions, not side effects of the new representation.

## Boundary matrix to complete

For each row, specify accepted value kinds, copy/share/transfer behavior, failure
diagnostic, publication point, cancellation cleanup and code/resource provenance.

| Boundary | Questions the contract must answer |
| --- | --- |
| Task spawn and task groups | How are argument graphs and callable environments admitted? |
| Task result and await | Who retains the result after the producer heap retires? Can multiple consumers observe it? |
| Channel send/receive and selection | Who owns queued messages and abandoned/selected cases? |
| Supervised retries | Which state survives a retry, and which attempt roots must disappear? |
| Synchronous callback | Does it execute on the caller's heap, and where are nested roots published? |
| Reusable callback session | Which heap survives between calls, and what happens on concurrent invocation? |
| Host export/import | Which operations retain a domain and which copy into another domain? |
| Debugger snapshot/edit | Is inspection detached data or a rooted live handle? How is mutation serialized? |
| Native blocking operation | What is rooted before execution ownership is released? |
| Session/executable boundary | How are callable code, globals and resource origins validated? |

## Transfer alternatives and required choice

**Graph copying** is the simplest candidate for transferable compound values.
Use a memoized traversal to preserve repeated references within one transferred
graph and to handle cycles if they are admitted. Copying must not accidentally
turn two shared mutable captures into independent cells inside the receiving
graph. Sender and receiver isolation, however, must be explicit: a receiver's
mutation must not silently update the sender's local heap.

**Ownership transfer** may avoid copies but needs proof that the sender has no
remaining local, host, debugger or queued references. Existing unrestricted
assignment syntax does not establish unique ownership. Do not claim zero-copy
transfer without such proof. It is optional, not a prototype prerequisite.

**Shared immutable storage** can avoid copying large values. Its graph cannot
reference local cells or retain hidden local roots. If resources are shared,
their synchronization and destruction rules belong to the resource contract.

Prefer the smallest coherent combination. Specify graph-copy work limits and
failure behavior; allocation failure or cancellation must not partially publish
a message or leak a destination graph. Determine lock ordering so transferring
in opposite directions cannot deadlock. Avoid holding two heap execution permits
across arbitrary host calls.

## Closure and aggregate semantics

Within a domain, closures capturing the same variable should initially continue
to observe one mutable cell. Returning from a routine removes stack roots; it
does not destroy reachable captures. Shared upvalues must remain shared after
their frame returns. Recursion and aggregates containing callables must work.

Decide whether cells are heap allocated immediately or stack backed until close.
Both are candidates; stack-backed cells require careful relocation/closing when
register storage grows, frames unwind or tasks suspend. Prefer the easier proof
unless measurements justify the additional machinery.

Specify observable array/record assignment and nested mutation behavior separately
from storage representation. Any semantic change needs before/after examples and
updated tests. Evaluate syntax changes on their merits, with rationale,
before/after examples, alternatives and a migration inventory. Record explicit
user approval before implementing the selected syntax; no legacy syntax mode
is required. Proposal examples remain clearly marked as unimplemented until then.

## Host handles and external access

Prefer opaque handles registered with the owning heap. A handle roots a value and
retains the necessary heap/code metadata. A scoped API temporarily acquires the
right to read or mutate it. Specify whether access from another thread blocks,
queues work or fails; a persistent handle alone never grants direct access.

Define handle cloning/release, stale-handle detection, shutdown with live handles,
cross-session import and the last-handle race. Candidate behavior: retiring a task
removes its execution roots but retained host handles keep their reachable graph
alive; releasing handles requests collection even when no task will run again.
Choose a maintenance mechanism that cannot lose release requests or deadlock
against callbacks. No compatibility promise requires today's constructor or
guard types to survive.

Native code must either publish roots and yield execution before a blocking wait,
or explicitly defer collection until it returns. Re-entry requires a deliberate
protocol. Finalizers cannot run arbitrary FPAS while sweep invariants are broken.
Explicit close/structured cleanup remains distinct from reclaiming heap memory;
deterministic file/channel cleanup must not accidentally become GC-timed.

## Acceptance and owning paths

Deliver the completed matrix, selected copy/share rules and host-access lifecycle.
Resolve every path by which a domain can retire with live exports. Specify
diagnostics for rejected transfers and failure paths for interrupted publication.
Include examples for shared local counters, a closure returning from its frame,
task results, queued messages, host retention and debugger access during suspension.

Likely owners: VM `tasks/`, `domain/transfer.rs`, `callback.rs`, `inspection/`,
runtime resource values, `fpas-sema` task-bound capability checks and the Rust
embedding APIs. Inventory exact paths before changing them. Current handbook
targets include closures, concurrency, variable references, embedding and debugger
documentation. AP48.9 records proposed behavior; AP48.11 updates current manuals
when the behavior is implemented.
