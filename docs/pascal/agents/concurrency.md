# Tasks, scopes and channels

Read this before starting concurrent work or passing values between tasks.

## Ownership and results

`go Work(Arguments)` is an expression: it returns `task of T` for a function
returning T, or bare `task` for a procedure. Bare task never infers a result type.
Every go requires a lexical `scope` in the same routine. Bind the handle or use
`discard go Work();`; a bare go expression statement is rejected. Called helpers
do not inherit their caller's scope ownership.

`scope ... end scope;` owns its lexically spawned children. Normal exit joins
them. Return, break, continue, failed try and runtime failure leaving a scope
cancel and join outstanding children before exit. Scope-owned handles cannot
escape. A returned `Error(...)` is ordinary Result data; a child panic fails the
scope. Discard does not detach, cancel, join or consume a child result.

Import `uses Std.Tasks as Tasks;` for wait, channel and cancellation APIs.
`Tasks.Wait(Handle)` retrieves the result once across all handle aliases;
procedure tasks use it as a statement. WaitAll/WaitAny observe homogeneous task
arrays without consuming retained results. TaskCase/Select support mixed typed
handles and other event sources.

## Transfers and channels

Workers cannot access foreign mutable variables, including globals reached
through helpers or callbacks. Pass snapshots, change worker-local copies, and
communicate through channels. Mutable-capture callables stay with their task:
they cannot cross through go arguments, channels or task results. Statically
known violations fail checking; dynamic targets retain runtime checks.
`go` cannot pass `var` arguments.

Channels carry a declared type. `CreateChannel(Capacity)` needs positive capacity;
Send and Receive may block and return Results. Close channels after their users
finish. Bounded queues need concurrent producers/consumers; joining a producer
before draining its full queue can deadlock.

```pascal id=owned-worker run
program AgentConcurrency;

uses Std.Tasks as Tasks, Std.Test as Test;

procedure Produce(Output: channel of integer);
begin
  Test.AssertTrue(Tasks.Send(Output, 42).Unwrap());
end procedure;

begin
  scope
    const Queue: channel of integer := Tasks.CreateChannel(1);
    const Worker: task := go Produce(Queue);
    const Value := Tasks.Receive(Queue).Unwrap();
    Tasks.Wait(Worker);
    Test.AssertEquals(42, Value);
    Test.AssertTrue(Tasks.CloseChannel(Queue));
  end scope;
end.
```

## Dynamic groups, cancellation and selection

Pass a TaskGroup when a helper needs to start caller-owned dynamic work. Group
workers take one by-value CancellationToken parameter and a fully instantiated
callable. Groups are cancelled and fully closed before their creating task
publishes its terminal outcome. Collect required data before closing; successful
close can still contain worker failures that the caller must inspect.

Cancellation is cooperative. Timed group close retains unfinished work on
timeout; the owner must later finish closing it. Supervision provides bounded
retries with explicit policy. These are library APIs, not extra syntax.
Controlled waits validate arguments/original failures, then check cancellation,
readiness and timeout. Select instead uses ready-case input order, including
timer and cancellation cases. Consult the exact operation before choosing a
timeout or shutdown policy.

Further rules: [go](../language/concurrency/go.md),
[task scopes](../language/concurrency/task-scopes.md),
[transfer rules](../language/concurrency/README.md), and
[Std.Tasks](../std/concurrency/task.md). For listener/worker lifetime and explicit
process escalation, read [Std.Server](../std/network/server.md).
