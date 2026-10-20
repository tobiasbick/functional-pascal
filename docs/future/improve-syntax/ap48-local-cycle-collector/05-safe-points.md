# AP48.5: Safe points and domain lifecycle

Package: [AP48](README.md). Status: not started. Effort: large.
Prerequisite: [AP48.4](04-roots-and-embedding.md) complete.

## Work

Define execution, external access, collection and retirement states for each domain.
An execution permit moves with the logical task. Exclusive collection excludes
execution, external graph access and changes to the registered root set. Do not
acquire a global interpreter lock or add a mutex operation to every VM instruction.

Place collection opportunities between completed dispatch operations, before parking
and during finalization. Account for the root returned by the last operation before
collecting. Native callback re-entry may reach a safe point only after all enclosing
native temporaries are registered. Define re-entry rules to avoid self-deadlock.

Prepare a root view under the owning storage protocol, then collect without holding
shared scheduler, channel or task-catalog locks. Specify lock ordering for per-domain
access, registries and cells. Never drop arbitrary retained values while holding a
cell lock. A busy or incomplete domain must defer collection rather than guess roots.

Retire execution roots after normal return, panic/failure, cancellation and structured
cleanup. First remove unused owner cases under AP48.2, then drop dead execution roots
and request collection. Explicit debugger/host roots may survive. On supervised
retry, close abandoned-attempt cases and discard attempt frames before admitting new
work; preserve the logical task's domain and legitimate external roots. Callback
invocation return keeps the session root domain active until shutdown/destruction.

## Maintenance execution

Use a per-domain monotonically advancing request generation and an exclusive permit.
Every external-root release and external edge-removing mutation records a request.
The thread making the request tries to service it synchronously if the domain is
quiescent. It first commits pending root removals and drops removed holders outside
registry/cell locks, then invokes the collector when automatic servicing is enabled
in AP48.7. AP48.6 first exercises the same route with explicit maintenance requests.
This includes retired domains with other surviving roots and idle callback sessions.
No original worker, live VM object, global maintenance lock or collector thread is
required. Domain handles own the remaining collector context without an ownership
cycle. The complete pass may contribute to host handle-release latency.

If execution or another accessor owns the permit, queue the request without waiting
on a permit already held by the same thread. The active owner services it at the
next eligible safe point. Outermost permit release must check pending generations
and hand off or perform maintenance before leaving a quiescent domain unserviced.
Requests racing with collection are handled in a subsequent pass. Specify the
generation/permit handshake so no request can arrive between the last check and
release and remain stranded. Host mutation follows the same rule as root removal.

## Blocking states

| State | Selected collection rule |
| --- | --- |
| Scheduler-suspended task | Publish all saved roots before parking; maintenance may acquire the domain without dispatching FPAS |
| Native wait with a published root view | Release execution access before waiting; maintenance may run on a requesting thread |
| Polling native wait | Service pending maintenance at explicit wait-loop safe points, with roots registered and resource locks released |
| Opaque or uninterruptible native operation | Retain execution access; defer requests until return/unwind; no prompt-reclamation guarantee |

For saved tasks, wakeup may record readiness during maintenance but cannot dispatch
the task until the permit is released. Take shared scheduler locks only to claim or
restore runnable ownership. For published native waits, reacquire execution access
before using local values, resuming FPAS or entering a callback. Publish native wait
roots and release access atomically with respect to maintenance. Registration alone
does not make unguarded native borrows safe.

Inventory native blocking entry points and classify each in this table. Include the
main-task channel loops, which do not reach ordinary dispatch boundaries while
waiting. Do not claim that all blocked tasks are independently collectible. Pending
requests must be serviced on return from an opaque call before the next FPAS batch.

## Owners and migration

Modify `crates/fpas-vm/src/vm/worker/run.rs`, worker save/restore, task pool and
suspension paths, callback re-entry and debug task drivers. Keep lifecycle and permit
logic in the domain module. Integrate failure capture and shutdown with that module.

## Acceptance and verification

- Controlled interleavings cannot run and collect one domain simultaneously.
- Migration, parking/wakeup and retries preserve the correct owner and roots.
- Collection of one domain allows an unrelated runnable task to make progress.
- Host access and root removal racing with collection serialize without lost roots,
  deadlock or references to reclaimed contents.
- Nested callbacks, last-operation results and pending cleanup retain live values.
- Terminal, blocked and externally retained domains have a demonstrated cleanup path.
- After task and VM destruction, releasing independent root A services its retired
  domain while root B remains alive. B remains valid; A does not wait for B's release.
- Requests arriving during collection or outermost permit release are not lost.
  Nested release does not recursively collect or wait on its own permit.
- A saved task remains parked during maintenance; a racing wakeup resumes it exactly
  once afterward. A published native wait remains blocked while collection completes.
- A polling main-task channel wait services maintenance without running queued tasks
  inline. An opaque native call defers it and services it on return/unwind. All native
  blocking entry points have a documented classification and regression owner.

Use existing scheduler/debugger synchronization test mechanisms and explicit barriers,
not arbitrary sleeps. First verify permit ownership, published roots and pending
request generations without reclamation. AP48.6 adds collection to this protocol.
Document lock order and lifecycle in the owning Rust modules when implemented.
