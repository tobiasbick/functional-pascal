# AP40.3: Wait contracts

Package: [AP40](README.md). Status: complete.

## Scope and prerequisites

Implement [AP40.1's recorded wait contract](01-concurrency-contract.md).
AP40.2 supplies the selected owner, teardown, and returned-Result failure model;
AP30 supplies fixed task types and the retained single-result-retrieval rule.

- Controlled Std.Tasks waits (task completion, channel operations, and applicable
  group-close operations) validate arguments and original task-failure diagnostics
  before ordinary outcomes. Every probe then prioritizes cancellation, available
  completion/transfer, and timeout expiry, wherever those outcomes apply.
- Use the same order on entry and after suspension. Timeout zero performs one
  immediate probe. Keep one monotonic deadline per invocation; wakeups/retries
  do not reset it. An observed ready outcome beats simultaneous expiry without
  claiming an unavailable physical completion timestamp.
- Cancellation/timeout ends the wait attempt rather than cancelling the observed
  task, closing a channel, consuming a result, or releasing group ownership.
  Group close still performs its explicitly requested cancellation/close action;
  a close timeout retains the group and its remaining children/results.
- Wait retrieves one logical completion once, including through handle aliases.
  WaitAll, WaitAny, controlled completion waits, and TaskCase observe completion
  without consuming results; observation after retrieval remains valid. Preserve
  the procedure-task Wait form without a public unit value or erased handle type.
- Returned Error values remain ordinary task results rather than task-failure
  diagnostics. Preserve original panic/runtime diagnostic precedence and recovery.
- Select retains first-ready case order, including cancellation and timer cases.
  These are explicit events; no implicit cancellation/timer sorting is added.
  Preserve validation/failure checks before transfer, exactly one callback, and
  no transfer/result consumption by losing cases.

Do not add a result-retrieval alias, new observer/linear type, or a second
first-probe policy. Keep the distinction between retrieval, observation, and
explicit event selection visible in API documentation and diagnostics.

## Delivered implementation

Controlled channel send/receive and task-completion probes use validation/original
failure checks followed by cancellation, readiness, and timeout. Entry, zero-timeout,
and resumed probes preserve the same order and one monotonic deadline. Group close
preserves its explicit cancellation, join, and retained-ownership behavior.
Select retains input case order without implicit cancellation/timer priority.

Manual-clock tests drive actual intrinsic entry and resumed suspensions with
simultaneous readiness/expiry and failure/cancellation/readiness. They verify no
channel transfer on cancellation, no completion consumption by observers, and
invalid identity precedence. Existing group-close/Select fixtures and the FPAS
owner-cleanup regression cover retained ownership, Result errors as data, and
observation after consuming Wait. Runtime and debugger checks agree; API and
authoring documentation describe the same contract.

Independent review has no remaining wait-contract blocker; see
[package verification](README.md#delivery-and-verification).

## Affected areas, migration, and documentation

Inspect hosted task/group/selection implementations under
`crates/fpas-vm/src/`, task builtins under `crates/fpas-std/src/`, and typed
call validation. Update [Std.Tasks](../../../pascal/std/concurrency/task.md)
and concurrency examples. Include task completion, channel send/receive, group
close, scheduler suspension/resumption, and deterministic debugger implementations
in the policy audit. Share the selected precedence without losing each operation's
commit/ownership rules.

Migrate callers and existing boundary fixtures that depended on timeout winning
over an already observed completion after suspension. Document the same order
for every applicable controlled API and explicitly document Select's case order.
Teach storing a consumed result once, retrieving it before owner cleanup, and
keeping handles for retrieval after non-consuming observation. Update compact
reference, diagnostics, authoring guidance, and AP30/AP26 cross-references.

## Verification

Use controlled readiness/cancellation signals and deterministic clock/probe
boundaries. Cover entry and resumed probes, timeout zero, readiness with expired
deadlines, cancellation with readiness/expiry, and original diagnostics with
competing ordinary outcomes. Test every applicable controlled API; no long sleep
or timing-only assertion is proof of priority.

Verify no commit on a cancelled/timed-out channel wait, retained ownership after
group-close timeout, unchanged results after completion observation, handle
aliases with repeated consuming waits, and returned Result errors as data.
Select tests deliberately vary ready cancellation/timer/task/channel case order
and verify one callback and no losing transfer. Exercise both runtime and debugger.

Run relevant component tests, Rust/FPAS suites, formatting, source-consumer checks,
documentation links/examples, and shared syntax regressions. Mark complete only
after runtime policy, migrated callers, and current API guidance agree.
