# AP48.2: Runtime transfer enforcement

Package: [AP48](README.md). Status: not started. Effort: large.
Prerequisite: [AP48.1](01-owned-values.md) complete.

## Work

Use the shared graph classification at every publication point. Validate all inputs
before admission or storage changes. Preserve source argument evaluation order;
failure prevents publication, not effects of arguments already evaluated.

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

Apply Q41 recursively to every imported function, including functions inside immutable
aggregates and bound receivers. A different executable token is a deterministic
invalid-callable import diagnostic before target execution. Two sessions sharing the
same verified executable may exchange otherwise transferable callables. Calls then
use target-session globals under the existing access rules. Same-code provenance
does not permit foreign cells, task-bound functions or foreign resource handles.

Implement Q38 owner/attempt cleanup in the selection registry in this package.
Remove unused cases after structured owner cleanup, and before publishing terminal
completion or admitting a retry. Already claimed cases belong to the pending Select
operation and are released by its cleanup. Neither path executes callbacks or closes
underlying channels/tasks. Same-owner `CloseWaitCase` on an automatically closed
handle returns false; selecting it fails as a closed case. Preserve wrong-owner and
invalid-kind diagnostics. Case identity is never reused by a later attempt.
Session shutdown/destruction closes session-owned cases; an invocation return alone
does not. AP48.4 registers these holders, and AP48.5 adds collection after removal.

## Owners and migration

Modify `crates/fpas-vm/src/vm/tasks/spawn.rs`, `tasks/groups/`, `tasks/supervision/`,
`tasks/channel/`, `tasks/selection/`, scheduler result publication, global execution
and `callback.rs`. Put shared policy in the domain module, not separate per-intrinsic
walkers. Verify ordinary, debug-driven and failure/retry execution paths.
Audit channel registry and scheduler callers so no lower-level publication bypass
remains. Migrate repository fixtures that depended on the documented prohibition
not being enforced. Final host interface migration belongs to AP48.4.

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

Extend existing VM task/channel/group/selection and global-access contract tests.
Use hand-built verified bytecode or host entry tests where necessary to exercise the
runtime guard independently of sema. Update the concurrency and closure handbook
with actual enforcement and diagnostics when delivered. Replace the Select handbook's
VM-teardown-only retention rule with Q38 in the same implementation delivery.
