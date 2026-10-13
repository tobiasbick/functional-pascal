# AP40: Explicit task lifetimes and shared state

Status: Complete (AP40.1–AP40.3).
Effort: large. Completion is tracked in the
[central plan](../README.md).

## Original findings addressed

- A direct spawn in a [scope](../../../pascal/language/concurrency/task-scopes.md)
  was owned, while named and anonymous helpers had different checks and unowned
  starts remained possible. Every go now requires a scope in its own routine.
- Mutable local captures could not cross tasks, but global variables were resolved
  outside the [capture environment](../../../pascal/language/functions/closures.md)
  and shared by workers. Foreign mutable access is now rejected; locked accesses
  never establish permission to share another task's mutable variables.
- [Task APIs](../../../pascal/std/concurrency/task.md) chose different
  ready/timeout/cancellation priorities. Wait consumes a result once, while
  WaitAll/WaitAny observe completion despite freely copyable handles.
- Task-group ownership survived an unused handle, but explicit discard was
  rejected by a type-based rule. Ordinary discard now accepts every value while
  keeping lifetime, initialization, and cross-task checks separate.

## Selected contract

Every task has an owner. Ownership and data transfer are explicit; extracting a
start into a helper cannot silently detach work or grant access to another
task's mutable variables.

- `go` starts a child of an enclosing lexical `scope` in the same routine.
  A start without that owner is rejected. Dynamic work uses an explicit
  `TaskGroup` through the existing group-start APIs. No unowned start form is
  retained or introduced.
- Named and anonymous helpers use the same boundary: they do not inherit the
  caller's scope. A helper opens its own scope or receives an explicit group.
  Lexical scopes and dynamic groups keep their distinct lifetime roles.
- `go` is always an expression returning `task` or `task of T`; its standalone
  statement form is removed. Bind the handle or explicitly discard the value,
  such as `discard go Audit(Input);` inside a scope. Discard ignores a value;
  it does not cancel, detach, join, or consume a task result. The owner retains
  the child and monitors its outcome independently of handle use.
- Apply the ordinary discard rule to task handles, aggregates, channels, and
  callable values as well. Remove discard-specific task-freedom restrictions,
  including generic and capture-proof exceptions. A procedure call without a
  value remains invalid as a discard operand. Ownership/escape restrictions
  and cross-task capture restrictions remain separate checks.
- A worker cannot read or write mutable variables belonging to another task,
  including program/unit globals and accesses through helpers or callbacks.
  Pass ordinary data as values and communicate through channels. Existing
  explicitly shareable resources, such as cancellation tokens, retain their
  specified operations. Worker-owned local mutation remains valid; AP37's
  writable-storage and InPlace rules remain intact.
- Normal scope completion joins its children. Early departure, panic, or a
  runtime error cancels and then joins remaining children. Open groups are
  cancelled and fully closed when their creating task ends. Dropping a handle
  cannot release ownership, and a waiting timeout cannot detach children.
- `Error(...)` is an ordinary Result value for every task owner. Groups no
  longer classify returned Error values automatically as task failures.
  Retrieve required results with Wait and handle them with existing try/case
  forms. Panic/runtime failures remain monitored independently of handle use;
  supervision may explicitly use returned Error values as retry criteria.
- Keep AP30's fixed types and single result retrieval: copies alias one task,
  and a second Wait for the same completion is a runtime error. WaitAll,
  WaitAny, and TaskCase observe completion without consuming results. Retrieve
  required results before their owner releases them and reuse the stored value.
- Controlled waits validate arguments and check task-failure diagnostics first.
  At every subsequent outcome probe, priority is cancellation, readiness, then
  timeout, including entry and timeout zero. Select retains explicit case-order
  selection; CancellationCase and TimerCase are selectable events there.

The full decision and verified example matrix are recorded in
[AP40.1](01-concurrency-contract.md). Do not add an effect system, a broad linear
type system, alternative spawn syntax, or an additional result-retrieval API.

## Replaced decisions and retained contracts

AP40 replaces AP26's unowned/go-statement behavior and automatic returned-Error
group failure reporting, extends its group teardown contract, and replaces
AP04's task-specific discard restrictions. AP26's lexical ownership boundary,
scope escape checks, and scope cancellation/failure rules are retained.
AP30's procedure/function task types, homogeneous observation APIs, and
single-consumption Wait contract are retained. These completed packages keep
their status; their current documentation now describes AP40 behavior.

## Open decisions

None for the delivered contract. AP40.2/AP40.3 include implementation, consumer
migration, current documentation, tooling, and regressions.

## Dependencies and work packages

AP26 and AP30 are complete. AP40.1 records the selected contract; AP40.2
implements ownership, discard, shared-state, and teardown changes together.
AP40.3 follows AP40.2 so controlled waits operate on the selected owner and
failure model. Preserve AP37 mutation and AP38 initialization checks.

- [x] [AP40.1: Concurrency contract](01-concurrency-contract.md)
- [x] [AP40.2: Ownership and shared state](02-ownership-and-shared-state.md)
- [x] [AP40.3: Wait contracts](03-wait-contracts.md)

## Delivery and verification

Parser/AST, IR, compiler, semantic analysis, VM, debugger, compiled-unit summaries,
source consumers, grammar, editor snippets, generated APIs, authoring guidance,
and language/API documentation implement the selected contract together.
The detached spawn path and discard-specific proof machinery are removed.
Bytecode version 21 and unit format 17 invalidate incompatible artifacts.

Statically known mutable-global dependencies are rejected with FP3042, including
imported and callback dependencies. Dynamically received/stored callable targets
retain mandatory VM global/reference access checks and task-bound capture checks;
unknown targets never authorize access to foreign mutable state. This preserves
stored callbacks without introducing public effect annotations.

Deterministic regressions cover owner publication after group join, main/debugger
cleanup, immediate and suspended cancellation, independent equal-source failures,
ordinary returned Result errors, controlled-wait precedence, and compiled-unit
reuse. An AP39 baseline probe failed because a parent result was published before
its children ended; the AP40 implementation passes the same probe.

The FPAS suite passes 511 tests with one skip; all 22 example/app projects and
98 standalone examples check successfully. All 609 VM tests outside the two
previously reproduced network timing/cancellation failures pass. The final full
Rust workspace run has only the unchanged socket-timeout rounding failure
(32ms observed versus 30ms requested), reproduced at baseline `03b98038`; its
other tests pass, including the second previously flaky network test.
Rust formatting and workspace build, release binaries and 111 standard-library
units, FPAS formatting, editor grammar, and 1,965 relative file links across
317 documents pass. Independent review findings in cleanup, diagnostic identity,
test fixtures, and documentation are fixed and rechecked; no AP40 blocker remains.

## Acceptance

Every start has a scope or group owner, with identical helper boundaries for
named and anonymous routines. Go has one expression form; discard has one
ordinary value rule. Foreign mutable state cannot be accessed across tasks.
Owners retain and finish children even after handle discard, and returned
Result errors remain data. Waiting APIs preserve single retrieval versus
completion observation and specify deterministic competing-event priorities.
