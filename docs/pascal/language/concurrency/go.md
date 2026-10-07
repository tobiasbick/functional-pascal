# `go`

Launch a concurrent task with the `go` keyword.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`go_call`, `go_stmt`, and the `go` alternative in `primary_atom`).

## Expression form (handle retained)

Use `go` as an expression and assign it to capture a `task` handle:

```pascal
program TaskExample;

uses Std.Console, Std.Tasks;

function Worker(): integer;
begin
  return 42;
end function;

begin
  const T: task := go Worker();
  const R: integer := Wait(T);
  WriteLn(R);
end.
```

## Statement form (fire-and-forget)

A `go` **statement** runs the call concurrently and **does not** produce a handle (the compiler discards the task result at the bytecode level). Use this when you only need side effects:

```pascal
go LogEvent('started');
```

## What `go` may target

`go` must be followed by a **single call expression** (not a bare designator or arbitrary value). The callee may be:

- a **function** or **procedure** (including qualified names such as `Std.Console.WriteLn(...)`),
- a **method** call, or
- a call through a **callable variable** (function type, procedure type, and similar).

Bare values, operators, and non-call expressions are rejected by the parser or semantic checker.

A `go` call cannot pass a `var` argument, and cannot start a nested routine that uses an enclosing
[`var` parameter](../functions/var-parameters.md) (FP3030): the task could outlive the caller's
variable.

## See also

- [Task handles](task-handles.md)
- [Scheduling](scheduling.md)
- [`Std.Tasks`](../../std/concurrency/task.md)
