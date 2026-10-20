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

Define retirement for normal return, panic/failure, cancellation, scope cleanup and
supervised restart. Terminal tasks keep only externally rooted graphs, not dead
execution frames. A final root release schedules or performs serialized cleanup
even if the original worker is gone. Apply the same rules to callback-session
shutdown and debugger evaluation domains. Blocked tasks need safe maintenance
opportunities so collection does not depend solely on future FPAS instructions.

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

Use existing scheduler/debugger synchronization test mechanisms and explicit barriers,
not arbitrary sleeps. Exercise protocol transitions without reclamation first.
Document lock order and lifecycle in the owning Rust modules when implemented.
