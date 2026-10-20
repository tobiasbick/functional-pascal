# AP48.4: Execution and external roots

Package: [AP48](README.md). Status: not started. Effort: large.
Prerequisite: [AP48.3](03-static-capabilities.md) complete.

## Work

Define one root enumeration interface for a domain. Complete the following inventory
against all `Value` and `SharedFunction` holders before declaring it exhaustive.

| Holder | Planned treatment |
| --- | --- |
| Active/saved registers and globals | Enumerate with their owning execution domain |
| Pending completion, suspension, retries and continuations | Enumerate retained values until completion or cleanup |
| Active/cached callback workers and suspended native callers | Include all frames and native temporary roots |
| Select registry and claimed cases | Register local callbacks and retained values by owner |
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
Debugger evaluation copies use a separate registered domain, including partial
construction and error cleanup. Failure capture must distinguish live retained
values from fully detached observations.

Define close/drop behavior for root handles. Administrative registries use weak
entries or another demonstrated acyclic ownership structure. A handle may outlive
task execution, but must not indefinitely retain unrelated unreachable cycles.

## Owners and migration

Modify `crates/fpas-vm/src/vm/inspection/`, `debug/`, `failure_capture/`,
`hosted/callbacks/`, `callback_call.rs`, `callback.rs` and selection storage.
Inventory public execution results and all embedding callers before finalizing the
handle interface. Runtime graph operations stay in the shared value modules.

## Acceptance and verification

- Enumerating roots covers each inventory row in active and suspended states.
- A host/debugger root preserves identity and contents across a simulated collection
  boundary; releasing it removes the root without a registry ownership cycle.
- Cross-session arguments cannot reuse numeric task identity as permission.
- Nested callbacks preserve the native caller's temporary values and frame roots.
- Detached evaluation success, failure and abandoned partial copies release their
  registrations. Live inspection retains no unregistered local `Value` copies.

Extend embedding, debugger, callback and failure-capture contract coverage. Update
public Rust documentation and migrate callers in the same delivery. Finish the
ownership milestone with AP48.1 through AP48.4 complete; collection remains disabled.
