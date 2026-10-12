# Task scopes

A `scope` block owns every task started directly inside it. The scope ends only after all of
its child tasks have finished, so no child outlives the block that started it.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`scope_stmt`).

```pascal
program ScopeExample;

uses Std.Console, Std.Tasks;

function Fetch(Id: integer): integer;
begin
  return Id * 10;
end function;

begin
  scope
    const A: task of integer := go Fetch(1);
    const B: task of integer := go Fetch(2);
    WriteLn(Wait(A) + Wait(B));
  end scope;
end.
```

The body is a nonempty statement list with its own local scope, closed by `end scope`.
`scope` is a reserved word.

## Ownership

Every `go` written directly in the scope body starts a child owned by the innermost scope,
whether it is a `go` expression that returns a handle or a `go` statement. A `go` outside any
scope has no lexical scope owner and is not joined automatically. Its expression form still
returns a handle that can be waited on; its statement form produces no handle. See [`go`](go.md).

Ownership never crosses a routine boundary:

- A routine called inside a scope does not inherit it. A `go` in that routine has no lexical
  scope owner unless the routine opens its own `scope`; handle retention is independent.
- A `go` in an anonymous routine that is written inside a scope is rejected (FP3038), because
  the anonymous routine may run after the scope has ended. Move the `go` into the scope body,
  or open a `scope` inside the anonymous routine.

```pascal
scope
  for Item: integer in Items do
    go Process(Item);
  end for;
end scope;
```

## Completion

| How the body is left | What the scope does |
| --- | --- |
| Normal arrival at `end scope` | Waits for every child without cancelling it |
| `return`, `break`, `continue`, or `try` propagation | Cancels the remaining children, then waits for them |
| Panic or runtime error | Cancels the remaining children, then waits for them |

Nested scopes are left innermost first. A routine's postconditions run after all of its
scopes have ended.

```pascal
function FirstMatch(Values: array of integer): integer;
begin
  for Value: integer in Values do
    scope
      go Audit(Value);
      if Value > 0 then
        return Value;
      end if;
    end scope;
  end for;
  return 0;
end function;
```

Here `return` cancels the `Audit` task of the current iteration before `FirstMatch` returns.

## Cancellation

Cancelling a child needs no token. The child stops at its next instruction boundary or at its
next wait point, such as `Wait`, a channel operation, `Select`, or `Std.Time.Sleep`, and ends
with the cancelled outcome (FP5016). A cancelled child also leaves every scope it has opened, so
its own children are cancelled as well.

A synchronous callback is part of its caller's execution and observes the same cancellation.
This does not give the callback ownership of the caller's scope: its `go` still follows the
routine-boundary rule above.

Cancellation is not a failure: it never fails the scope, and a cancelled task's result is
discarded when the scope ends. `CancellationToken` values remain the mechanism of
[task groups](../../std/concurrency/task.md#task-groups); scopes do not create them.

## Failures

A child **panic** or other **runtime error** fails its scope:

1. The siblings and the scope body are cancelled at their next instruction boundary or wait
   point.
2. The scope waits for every child.
3. Leaving the scope re-raises the original diagnostic of the failure.

The first failure observed, from the body or from a child, is the primary diagnostic. Every
further failure is listed in its help text as `Further failures in this task scope`. When the
body is itself waiting for the failed child, `Wait` reports the same failure; it is reported
once. A failure takes precedence over `return`, `break`, `continue`, and `try` propagation.
Independent failures remain distinct even when their messages and source positions are equal.

A child that returns `Error(...)` has **not** failed: its `Result` is an ordinary value. Keep
the handle and forward the error explicitly:

```pascal
function LoadBoth(Left: string; Right: string): result of (string, string);
begin
  scope
    const A: task of result of (string, string) := go Load(Left);
    const B: task of result of (string, string) := go Load(Right);
    const First: string := try Wait(A);
    const Second: string := try Wait(B);
    return Ok(First + Second);
  end scope;
end function;
```

If `try Wait(A)` propagates an error, `B` is cancelled before `LoadBoth` returns.

A `go` statement inside a scope whose call returns `result of ...` is rejected (FP3039): the
statement form has no handle, so its error would be lost.

## Handle escapes

A scope ends only after its children, so their handles must stay inside it. A handle of a
scope-owned child may appear only:

- in bindings declared inside the scope, including arrays, records, and other values built
  there;
- in `Std.Tasks` wait operations such as `Wait`, `WaitAll`, and `WaitAny`;
- as an argument of a direct call to a routine whose parameter does not escape, including a
  `go` of a sibling task.

Everything else is rejected with FP3040, which names the owning scope:

- `return` of the handle;
- assignment to a binding declared outside the scope, including through a `var` parameter, a
  field, or an element;
- sending it through a channel;
- capturing it in an anonymous routine;
- passing it to a call through a callable value, or to a parameter that can escape.

This also rejects native operations such as `Jobs.ForEach(Callback)` when they would pass
scope-owned handles to a callback through a callable value.

```pascal
function Total(Jobs: array of task of integer): integer;
begin
  var Sum: integer := 0;
  for Job: task of integer in Jobs do
    Sum := Sum + Wait(Job);
  end for;
  return Sum;
end function;

begin
  scope
    var Jobs: array of task of integer := [];
    Jobs.Push(go Fetch(1));
    Jobs.Push(go Fetch(2));
    WriteLn(Total(Jobs));
  end scope;
end.
```

A parameter escapes when its value, or a value built from it, can be returned, stored outside
the routine, sent through a channel, captured by an anonymous routine, passed to a callable
value, or passed on to another escaping parameter. The compiler infers this for every routine
without annotations and records it in the [unit interface](../../program-structure/projects.md),
so routines imported from other units are checked the same way. `Total` above only waits for its
argument, so it accepts scope-owned handles; a routine that stores a handle in a unit variable
does not. There is no runtime fallback: what the compiler cannot prove is rejected.

## Task scopes and task groups

| | `scope` | [Task group](../../std/concurrency/task.md#task-groups) |
| --- | --- | --- |
| Lifetime | Lexical block | Value created and closed explicitly |
| Cancellation | Automatic, no token | `CancellationToken` checked by the worker |
| Failure | Re-raised when the scope ends | Reported by `CloseTaskGroup` |
| Passing ownership between routines | Not possible | Pass the `TaskGroup` |

Use a task group for dynamic cases such as passing ownership between routines or supervised
retries.

## Debugging

The debugger shows scope children as ordinary tasks. Forced return and frame restart are
rejected while a discarded or restarted frame has an open task scope; step past `end scope`
first. Cancelling a task through the debugger also cancels and joins its open scopes after
continue, before that task publishes its cancelled outcome.

## See also

- [`go`](go.md)
- [Task handles](task-handles.md)
- [Scheduling](scheduling.md)
- [Panic](../error-handling/panic.md)
