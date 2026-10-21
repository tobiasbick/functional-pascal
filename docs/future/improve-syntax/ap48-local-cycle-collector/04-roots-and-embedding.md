# AP48.4: Execution and external roots

Package: [AP48](README.md). Status: complete. Effort: large.
Prerequisite: [AP48.3](03-static-capabilities.md) complete.

## Work

Define one root enumeration interface for a domain. Complete the following inventory
against all `Value` and `SharedFunction` holders before declaring it exhaustive.

| Holder | Planned treatment |
| --- | --- |
| Active/saved registers and globals | Enumerate with their owning execution domain |
| Pending completion, suspension, retries and continuations | Enumerate retained values until completion or cleanup |
| Active/cached callback workers and suspended native callers | Include all frames and native temporary roots; publish stable roots before eligible native waits |
| Select registry and claimed cases | Register by owner and attempt; remove roots when AP48.2 owner/attempt cleanup closes cases |
| Debugger inspection and failure capture | Retain registered live roots or detached snapshots |
| Host-held local values | Export registered handles with domain-controlled access |
| Channel messages and published task results | Verify AP48.2 guarantees no local graph is retained |

Replace raw local-value export with registered host/debugger handles. A handle keeps
the graph alive but grants no unsynchronized access to its cells. Reads, mutation,
cloning roots and session re-entry follow the domain protocol. Keep ordinary
transferable outputs convenient. Update all embedding consumers with this change;
do not retain an untracked compatibility path for local values.

Native operations must register local temporary values before calling FPAS again or
reaching a collection point. Hidden Rust locals are not automatically roots.
Registering temporaries alone does not make a blocking call collectible. Q40 also
requires publishing the complete suspended root view and releasing execution access.
Native code must retain no unguarded cell payload access across that transition.
Debugger evaluation copies use a separate registered domain, including partial
construction and error cleanup. Failure capture must distinguish live retained
values from fully detached observations.

Define close/drop behavior for root handles. Administrative registries use weak
entries or another demonstrated acyclic ownership structure. A handle may outlive
task execution, but must not indefinitely retain unrelated unreachable cycles.

Implement Q39 request generation for every external-root removal, including a
non-final removal, and for host mutations that disconnect graph edges. Keep root
removal and maintenance notification in one protocol. A deferred removal remains
a root until that protocol commits it. AP48.5 supplies the execution mechanism;
AP48.4 tests request delivery without enabling reclamation.

Root handles retain enough domain state for tracing after the original task, VM or
session object has been destroyed. They do not keep dead execution frames or the
whole hosted registry alive. A retired handle permits registered inspection, guarded
value mutation and release, but cannot re-enter FPAS after its session has shut down.
Mutation still validates ownership and requests maintenance for disconnected values;
it cannot invoke callbacks or revive closed host resources.
Root removal within a current permit queues work for its outermost release rather
than recursively taking the permit. Unrelated external roots remain registered.

Host ingress applies Q41 before creating target-session roots. Detached debugger
copies keep their code provenance while receiving fresh cell-domain identity.
Any debugger executable replacement must revalidate/rebind function provenance via
the existing code-mapping contract or reject the value; copying a numeric ID is not
rebinding. Local values never become transferable merely by sharing code identity.

## Owners and migration

Modify `crates/fpas-vm/src/vm/inspection/`, `debug/`, `failure_capture/`,
`hosted/callbacks/`, `callback_call.rs`, `callback.rs` and selection storage.
Inventory public execution results and all embedding callers before finalizing the
handle interface. Runtime graph operations stay in the shared value modules.

## Implementation scope

- Add shared root tracing, weak registrations, guarded external access and deferred
  removal/request-generation primitives under `crates/fpas-bytecode/src/value/roots/`.
  Extend `value/domain.rs`, `value/mod.rs` and `lib.rs`; cover independent roots,
  retired domains, nested access and edge-removing mutation without collection.
- Add `crates/fpas-vm/src/vm/domain/roots.rs` and focused tracing implementations
  for `worker.rs`, `tasks/{state,suspension}.rs`, supervision inputs, callback
  continuations and selection cases. Trace active and saved execution through the
  same interface; document every retained Value/SharedFunction owner in a focused
  root inventory beside the domain implementation.
- Migrate `vm/host_value/`, `callback.rs`, `callback_call.rs`, `mod.rs` and `run.rs`
  to registered exports, guarded inspection/mutation and rooted idle globals.
  Preserve sealed provenance, same-session identity, pure-data convenience and
  shutdown rejection. Add host lifecycle and registration regressions.
- Register native caller/temporary graphs in `hosted/callbacks/`, callback re-entry,
  `tasks/selection/` and their cleanup paths. Use the shared callback operation
  representation for complete native-state tracing; keep native blocking opaque
  until AP48.5 publishes execution access.
- Migrate retained inspection, debug handle/evaluation/call state, trace histories
  and live failure observations under `inspection/`, `debug/` and `failure_capture/`.
  Detached diagnostic trees remain root-free observations. Retain the sandbox
  domain through returned inspection values and partial/error cleanup.
- Validate contained callable targets during debugger executable replacement.
  Reject retained callable graphs before commit when the existing instruction mapping
  cannot atomically rebind their provenance. Modify `debug/live_image/` and shared
  detachment helpers as required; never authorize a numeric target alone.
- Update `docs/pascal/program-structure/runtime-ownership.md`, embedding/debugger
  guidance and relevant Rust API docs. Confirm the compact language reference;
  no syntax, safe-point servicing or reclamation is introduced.

## Acceptance and verification

- Enumerating roots covers each inventory row in active and suspended states.
- A host/debugger root preserves identity and contents across a simulated collection
  boundary; releasing it removes the root without a registry ownership cycle.
- Cross-session arguments cannot reuse numeric task identity as permission.
- Root A's release requests maintenance while independent root B remains alive,
  including after the session object is destroyed. Removal from inside a permit is
  deferred without self-deadlock or losing the request.
- Owner/attempt case cleanup removes the corresponding registry roots. Other live
  cases and explicitly retained debugger roots continue to preserve their values.
- Shutdown prohibits re-entry through surviving handles while keeping inspection,
  guarded value mutation and release valid. Code provenance survives detachment
  and rejects foreign imports.
- Nested callbacks preserve the native caller's temporary values and frame roots.
- Detached evaluation success, failure and abandoned partial copies release their
  registrations. Live inspection retains no unregistered local `Value` copies.

Extend embedding, debugger, callback and failure-capture contract coverage. Update
public Rust documentation and migrate callers in the same delivery. Finish the
ownership milestone with AP48.1 through AP48.4 complete; collection remains disabled.

## Delivered implementation notes

The [holder inventory](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md) maps retained
values to shared tracing, weak registrations or detached observations. Host values,
including debugger termination results, use sealed registered handles. Clone/removal,
nested external access, retired mutation and selection cleanup follow request generation.
Debugger result handles retain their detached domain; operation-scoped handoffs and
partial construction release registrations on error as well as success.

Live-image replacement rejects retained callable graphs before commit. Its existing
instruction mapping is not an atomic callable-graph rebind. Invocation and debugger
calls validate the original opaque executable identity; a numeric ID is insufficient.

Execution exclusion, eligible wait publication, request servicing and reclamation
remain AP48.5/AP48.6 work. No collector is active in this milestone.

## Regression coverage

Root tests cover independent clones and non-final releases, nested/deferred removal,
busy providers, retired domains, guarded mutation and maintenance request delivery.
VM coverage includes initialized and saved register state, native caller retention,
selection claim/owner cleanup, failure callback frames, independent host clones and
shutdown before first invocation. Debugger regressions cover detached child-domain
retention, abandoned partial copies, bounded nesting, operation handoff success/error
cleanup, registered terminal exports and atomic rejection of unrebound live images.

The compact language reference was confirmed against this delivery. No FPAS syntax
or language-facing invocation semantics changed; the Rust embedding ownership
contract and debugger lifetime rules are documented in the current handbook.
