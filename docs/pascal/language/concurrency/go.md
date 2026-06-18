# `go`

Launch a concurrent task with the `go` keyword.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`go_expr`, `go_stmt`).

## Expression form (handle retained)

Use `go` as an expression and assign it to capture a `task` handle:

```pascal
uses Std.Console, Std.Task;

function Worker(): integer;
begin
  return 42
end;

begin
  var T: task := go Worker();
  var R: integer := Wait(T);
  WriteLn(R)
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

## See also

- [Task handles](task-handles.md)
- [Scheduling](scheduling.md)
- [`Std.Task`](../../std/concurrency/task.md)
