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
| Host/session ingress | Incoming graph and domain identity | Reject before entering the target session |

Selection callbacks remain local roots and may be task-bound; do not reject them
as transferred work merely because a send payload must be transferable. Resource
handles keep their own existing lifetime and ownership checks. Same-task invocation
remains valid. Do not add implicit copying or ownership reassignment.

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

Extend existing VM task/channel/group/selection and global-access contract tests.
Use hand-built verified bytecode or host entry tests where necessary to exercise the
runtime guard independently of sema. Update the concurrency and closure handbook
with actual enforcement and diagnostics when delivered.
