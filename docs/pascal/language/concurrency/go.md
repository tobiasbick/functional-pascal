# `go`

`go Call(...)` starts one child in the innermost lexical `scope` of the same
routine. It is an expression returning `task of T` for a function returning T,
or `task` for a procedure. Bind the handle or explicitly discard it.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`go_call` and the `go` alternative in `primary_atom`).

## Starting owned work

Use `go` as an expression and assign it to capture a handle: `task of T` for
a function returning T, or `task` for a procedure:

```pascal id=example-1
program TaskExample;

uses Std.Console as Console, Std.Tasks as Tasks;

function Worker(): integer;
begin
  return 42;
end function;

begin
  scope
    const T: task of integer := go Worker();
    const R: integer := Tasks.Wait(T);
    Console.WriteLn(R);
    discard go Worker();
  end scope;
end.
```

Standalone `go Work();` is an unused-value error (FP3022). Use
`const Job := go Work();` or `discard go Work();` inside a scope. Discard
does not cancel, detach, join, or consume the result; the scope retains the child
and monitors panic/runtime failures. Returned `Error` values are ordinary data.

Starting outside a same-routine scope is FP3038. Named and anonymous helpers
do not inherit the caller's scope. A helper opens its own scope for work that
finishes there, or receives an explicit TaskGroup and uses StartTaskInGroup
for caller-owned dynamic work. See [task scopes](task-scopes.md) and
[groups](../../std/concurrency/task.md#task-groups).

## What `go` may target

`go` must be followed by a **single call expression** (not a bare designator or arbitrary value). The callee may be:

- a **function** or **procedure** (including alias-qualified calls such as
  `Console.WriteLn(...)` after `uses Std.Console as Console;`),
- a **method** call, or
- a call through a **callable variable** (function type, procedure type, and similar).

Bare values, operators, and non-call expressions are rejected by the parser or semantic checker.

A `go` call cannot pass a `var` argument, and cannot start a nested routine that uses an enclosing
[`var` parameter](../functions/var-parameters.md) (FP3030): the task could outlive the caller's
variable. Implicit writable collection InPlace receivers follow the same rule: `go Items.PushInPlace(Value)`
and `go Items.PopInPlace()` are rejected. Call these operations on the current task.
Value operations, including Push and Pop, may run in tasks and preserve the receiver.

Workers cannot read or write foreign mutable variables, including program/unit
globals through helpers and callbacks (FP3042 for statically known access).
Inputs are evaluated on the starting task: pass values and mutate worker-local
copies, or communicate through channels. Dynamic callable targets retain the
VM's mandatory foreign-global and task-bound-capture checks. Mutable globals
never become permitted merely because a callable's dependencies are unknown.

## See also

- [Task handles](task-handles.md)
- [Task scopes](task-scopes.md)
- [Scheduling](scheduling.md)
- [`Std.Tasks`](../../std/concurrency/task.md)
