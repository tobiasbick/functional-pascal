# AP40.1: Concurrency contract

Package: [AP40](README.md). Status: complete; agreed contract recorded.
The recorded decision is implemented in
[AP40.2](02-ownership-and-shared-state.md) and [AP40.3](03-wait-contracts.md).

## Ownership and start form

Every started task has exactly one owner: a lexical scope for go, or an explicit
TaskGroup for group/supervised starts. Unowned starts are rejected. A retained
handle identifies a child; copying or dropping that handle does not transfer
or release ownership.

A scope owner is available only within the routine containing the start.
Named and anonymous helpers never inherit the caller's scope. They open their
own scope or use a TaskGroup passed explicitly by the caller. A worker's own
scope owns its nested children. Preserve existing scope-handle escape checks.

Go has one expression form and returns AP30's fixed handle type: task for a
procedure, task of T for a function. Remove the separate go statement.
Existing group-start operations retain their explicit Group and worker
arguments; no new keyword or alternative start API is introduced.

## One discard rule

Discard evaluates a value expression once and ignores that value. It applies
to task handles and values containing them, including generic values and
callables with captures. Remove only discard-specific task-freedom checks;
scope escapes, initialization checks, and cross-task callable restrictions
continue to apply. Procedure calls with no value cannot be discard operands.

Discarding a task handle does not wait, cancel, detach, or take the task's result.
The owner continues to retain the child, monitor failures, and release retained
results at completion/close. Other aliases remain valid under the ordinary
single-retrieval rule. Explicit discard can intentionally ignore a future
Result value, just as ordinary discard can ignore an existing Result value.

The following fragment uses the implemented AP40.2 rules:

```pascal
scope
  discard go Audit(Input);
  const Job := go Compute(Input);
  const Value := Wait(Job);
  Show(Value);
end scope;
```

## Data and mutable state

Worker tasks cannot read or write another task's mutable variables, whether
those variables are local captures or program/unit globals. The restriction
includes transitive helper/callback access and applies to named, anonymous,
group, and supervised workers. Worker-owned local variables remain mutable.

Evaluate input values in the starting task and pass them as ordinary values.
Use channels for inter-task communication. Existing explicitly shareable
resource handles, including cancellation tokens and group handles used through
their specified operations, keep their resource semantics. Do not introduce
general shared mutable cells, implicit locking, public effect annotations, or
a broad linear type system. Individual VM access locks do not prove that
Counter := Counter + 1 is atomic.

## Completion and failure

- Normal scope exit waits for all owned children.
- Return, break, continue, try propagation, panic, and runtime error cancel and
  then join remaining scope children, preserving AP26's ordering and diagnostics.
- Groups remain explicit dynamic owners. They are created and closed by their
  owning task; helpers receive the group when registering work. Any group still
  open when that task terminates is cancelled and fully closed as part of its
  teardown. Preserve registered failure diagnostics during cleanup.
- Timeouts end a waiting attempt; they do not establish that children stopped,
  release ownership, or replace final cleanup. Cancellation/join is cooperative.
- Returned Error values are ordinary Result data, including for groups. Remove
  automatic returned-Error group failure reports. Handle required results with
  Wait plus existing try/case forms before the owner releases those results.
- Panic/runtime failures remain monitored even when all user handles are
  discarded. Cancellation itself does not become an unexpected worker failure.
- Supervision may explicitly retry on returned Error values. Its final returned
  Result still follows the ordinary data rule; exhaustion does not turn Error
  into an implicit runtime failure.

## Waiting and observation

Keep AP30's distinction: Wait retrieves the result once, while WaitAll, WaitAny,
and TaskCase only observe completion. Copies refer to the same logical task;
Wait through a second alias after retrieval is an error. Store the retrieved
value and reuse it. No erased task type or second result-retrieval API is added.

Controlled Std.Tasks waits validate arguments and check original task-failure
diagnostics before ordinary outcomes. Each probe then uses:

1. cancellation, when that operation observes cancellation;
2. available completion/transfer;
3. expired timeout, when that operation has a deadline.

The order is the same on entry and on later probes. Timeout zero permits one
immediate probe in this order. Deadlines are monotonic and captured once per
invocation; readiness wins when observed together with expiry, without claiming
that the event physically occurred before the deadline.

Select remains ordered event selection. The first ready case in input order
wins, including CancellationCase and TimerCase. Those cases report selectable
events rather than imposing the controlled operation's implicit priority.
Existing source validation, failure checks, and exactly-one-callback semantics
are preserved.

## Decision example matrix

These outcomes are implemented and covered by AP40.2/AP40.3 regressions.

| Situation | Selected outcome |
|-----------|------------------|
| Go expression in a scope in the same routine | Accept; the innermost scope owns the child |
| Go expression without a lexical scope | Reject; open a scope or start through an explicit group |
| Named/anonymous helper called from a scope starts go without its own scope | Reject equally; no inherited ownership |
| Helper uses an explicitly passed TaskGroup | Accept under group registration rules |
| Standalone go Audit(Input); | Reject; bind the go expression or write discard go Audit(Input); |
| Discard a scope/group task handle or an aggregate containing handles | Accept subject to ordinary expression and separate lifetime checks |
| Discard a procedure call that produces no value | Reject; call the procedure directly |
| Worker reads/writes a mutable global through an imported helper | Reject as foreign mutable state access |
| Worker mutates its own local copy of a supplied value | Accept under ordinary mutation rules |
| Discarded child's panic/runtime error | Owner still monitors and reports the failure |
| Group child returns Error(Message) | Ordinary Result data; no automatic group failure record |
| Creating task ends with an open group | Cancel and fully close the group during teardown |
| Wait through two aliases of one task | First retrieves; second reports already consumed |
| WaitAll/WaitAny/TaskCase followed by Wait | Observation leaves the result available for retrieval |
| Invalid arguments or task failure plus cancellation | Original diagnostic precedes ordinary outcomes |
| Cancellation plus readiness in a controlled wait | Cancellation wins without consuming an available result/transfer |
| Readiness plus timeout expiry at any probe | Readiness wins |
| Select has several ready events, including cancellation/timer | First ready input case wins |

## Documentation boundary

The current [scope](../../../pascal/language/concurrency/task-scopes.md),
[capture](../../../pascal/language/functions/closures.md),
[discard](../../../pascal/language/functions/discard.md), and
[task API](../../../pascal/std/concurrency/task.md) pages describe the delivered
contract. Known dependencies are checked statically; dynamic callable targets
retain mandatory VM access and capture checks rather than granting foreign access.
AP40 explicitly replaces the affected AP04/AP26 decisions while preserving
their completion status and AP30's fixed result types and retrieval contract.
No decision remains open for the selected scope.
