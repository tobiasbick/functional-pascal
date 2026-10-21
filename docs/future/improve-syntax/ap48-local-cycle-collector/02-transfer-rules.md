# AP48.2: Runtime transfer enforcement

Package: [AP48](README.md). Status: complete. Effort: large.
Prerequisite: [AP48.1](01-owned-values.md) complete.

## Delivered contract

The shared graph classification validates every publication point before admission
or storage changes. Source argument evaluation order is preserved; failure prevents
publication, not effects of arguments already evaluated.

| Operation | Values to check | Required failure behavior |
| --- | --- | --- |
| `go` | Callable, receiver, captures and all arguments | No child admitted or result slot left pending |
| Group start and supervised restart | Work function and retained retry inputs | No invalid attempt admitted |
| Send, TrySend, timeout and cancellation variants | Complete payload graph | No message queued or delivered |
| Select send cases | Payload when retained and before committed send | No invalid payload published; preserve case consumption contract |
| Task result | Complete returned graph | Publish the existing task-failure path, not a successful forbidden result |
| Foreign global load | Stored graph, including immutable globals | No foreign local graph exposed |
| Host/session ingress | Incoming graph, domain identity, executable provenance and resource handles | Reject before entering the target session |

Selection callbacks remain local roots and may be task-bound; do not reject them
as transferred work merely because a send payload must be transferable. Resource
handles keep their own existing lifetime and ownership checks. Same-task invocation
remains valid. Do not add implicit copying or ownership reassignment.

Session imports apply Q41 recursively to every function, including functions inside immutable
aggregates and bound receivers. A different executable token is a deterministic
invalid-callable import diagnostic before target execution. Two sessions sharing the
same verified executable may exchange otherwise transferable callables. Calls then
use target-session globals under the existing access rules. Same-code provenance
does not permit foreign cells, task-bound functions or foreign resource handles.

The selection registry implements Q38 owner/attempt cleanup. Unused cases are removed
after structured owner cleanup and before publishing terminal completion or admitting
a retry. Already claimed cases belong to the pending Select operation and are
released by its cleanup, including debugger disconnect. Neither path executes
callbacks or closes underlying channels/tasks. Same-owner `CloseWaitCase` on an
automatically closed handle returns false; selecting it fails as a closed case.
Wrong-owner and invalid-kind diagnostics remain unchanged. Case identity is never
reused by a later attempt.
Session shutdown/destruction closes session-owned cases; an invocation return alone
does not. AP48.4 registers these holders, and AP48.5 adds collection after removal.

## Owners and migration

Modify `crates/fpas-vm/src/vm/tasks/spawn.rs`, `tasks/groups/`, `tasks/supervision/`,
`tasks/channel/`, `tasks/selection/`, scheduler result publication, global execution
and `callback.rs`. Put shared policy in the domain module, not separate per-intrinsic
walkers. Verify ordinary, debug-driven and failure/retry execution paths.
Audit channel registry and scheduler callers so no lower-level publication bypass
remains. Migrate repository fixtures that depended on the documented prohibition
not being enforced. Introduce the sealed host export and argument interface here:
resource registries can reuse numeric IDs across sessions, so admission cannot prove
resource provenance from a raw `Value`. AP48.4 adds root registration and controlled
access behind the same interface.

## Implementation scope

- Extend `crates/fpas-bytecode/src/value/graph.rs` with an observation mode that
  reuses the iterative traversal while visiting resource-shaped records even when
  ownership summaries are neutral. Export the API through `value/mod.rs` and
  `lib.rs`, and add traversal regressions.
- Expose iterative value release through the bytecode value API and make
  `value/payload.rs` clone nested Option/Result wrappers iteratively. Host exports
  and rejected publications must release deep graphs without recursive Rust
  teardown, including ordinary drops of wrapper and function capture chains in
  `value/{payload,function}.rs`. Route aggregate buffer recycling in
  `value/managed_heap.rs` through the existing iterative drainer as well, so raw
  deep arrays, records, enums and dictionaries are safe at the same boundaries.
- Add shared transfer policy under `crates/fpas-vm/src/vm/domain/`. Guard spawn,
  group admission, supervised retries, channel publication, selection send cases,
  task results and foreign global reads at their owning boundaries. Audit lower-level
  registry and scheduler entry points and add focused rejection/atomicity tests.
- Modify `tasks/selection/registry.rs` and task completion/retry/session shutdown
  paths for owner cleanup. Keep claimed cases owned by their pending operation;
  drop removed callbacks outside the registry lock.
- Migrate `vm/channels/registry.rs` and its focused ownership tests for lower-level
  sends. Extend `debug/session/{forced_return,completed_result}.rs` and
  `debug/tasks/driver/completion.rs` so rejected debugger replacements cannot
  publish local child results or change terminal state prematurely.
  Clear cancelled pending operations in `debug/tasks/driver.rs` so debugger
  disconnect releases already claimed Select callbacks while the session survives.
  Migrate the older cell-capturing assignment fixtures using a test-only stopped
  binding injector in `debug/session/mutation.rs`; forbidden child results can no
  longer be used to construct those invocation/assignment test inputs.
- Add a focused `vm/host_value/` interface with sealed `HostValue`,
  `HostArgument::{Data, Export}`, session identity and admission checks. Migrate
  public `Execution`, `Vm::call`, `CallbackSession::invoke`, `vm/run.rs`,
  `vm/worker/run.rs`, `callback_call.rs`, failure-capture result consumers and
  `crates/fpas-vm/src/lib.rs`. Preserve raw values only in the internal execution
  result. Data extraction checks the complete graph; child selection preserves
  export provenance. Root registration and maintenance remain AP48.4 work.
- Migrate embedding fixtures in VM, compiler and linker tests. Add shared-image,
  foreign-image, nested callable, own-resource and foreign-resource admission
  regressions, including HTTP handle records and closed resource roundtrips.
  Put source-defined HTTP integration coverage in
  `crates/fpas-build/tests/host_provenance.rs`, using the actual standard-library
  project and public callback interface rather than builtin-only compiler fixtures.
- Guard retained HTTP states in `vm/hosted/http_handles.rs`, which are shared
  registry values, using the same transfer policy. Add focused regressions in
  `vm/hosted/http_handles/ownership_tests.rs` and
  `vm/execute/aggregates/global_tests.rs` for HTTP publication and global loads.
- Update the closure, concurrency and Select-lifetime handbook and
  `docs/pascal/program-structure/runtime-ownership.md`; confirm the compact reference.

## Acceptance and verification

- Transfer tests cover each table row with direct and aggregate-wrapped local values.
  Include ordinary helper arguments and returned closures with unknown static origin.
- Immutable data, closures with genuinely transferable captures, and valid handles
  still cross task boundaries and produce the same results.
- Rejected sends preserve channel contents. Rejected starts preserve task accounting.
  Rejected task results complete as failures and do not strand waiters or scopes.
- An immutable global containing a local closure is inaccessible to foreign tasks,
  while use by its owner remains valid.
- Select callbacks retain their same-task behavior. Timeout/cancellation priority and
  successful result consumption remain unchanged.
- A task ending with an unused mutable callback removes its case entry on success,
  failure and cancellation. An abandoned supervised attempt removes its cases before
  retry. Stale case copies cannot select a new attempt's case, and cleanup never
  invokes the callback or closes its resource. Collection of cycles is tested later.
- Sessions with matching numeric function IDs but different code reject import,
  including nested and bound callables. Sessions sharing one executable accept a
  transferable callable and observe target-session globals. Foreign captures and
  handles remain rejected even when executable identity matches.

VM task/channel/group/selection and global-access contract tests cover these
boundaries. Hand-built verified bytecode and host entry tests exercise runtime
guards independently of sema, including deep rejection on a small Rust stack.
The concurrency, closure, Select-lifetime and Rust embedding handbooks describe
the delivered enforcement. Registered root lifetime and controlled graph access
remain AP48.4 work; collection remains disabled.
