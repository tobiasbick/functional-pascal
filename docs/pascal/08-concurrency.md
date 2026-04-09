# 8. Concurrency

Functional Pascal provides Go-inspired lightweight task concurrency. Tasks created with `go` execute in parallel across multiple OS threads automatically.

## Tasks

Launch a concurrent task with the `go` keyword:

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

`go` accepts only a function or procedure call expression. Bare values and other expressions are rejected by the compiler. The VM distributes tasks across a thread pool for true parallel execution. The pool size equals the number of available CPU cores.

### Task Type

The `task` type represents a handle to a running task. Assign the result of a `go` expression to capture it. For type checking, the handle keeps the spawned call's result type, while the runtime value is an opaque task handle:

```pascal
var T: task := go ComputeSomething(Data);
```

## Task Management

### Waiting for a Task

`Std.Task.Wait` blocks until the task completes and returns its result:

```pascal
var T: task := go Compute(100);
var Result: integer := Wait(T);
```

### Waiting for Multiple Tasks

`Std.Task.WaitAll` blocks until all tasks in the array complete:

```pascal
WaitAll([T1, T2, T3]);
```

## Fork-Join Pattern

The idiomatic way to run parallel work is to spawn one task per unit of work and then wait for all results:

```pascal
program ParallelSum;
uses Std.Console, Std.Task;

function Compute(N: integer): integer;
begin
  return N * N
end;

begin
  var T1: task := go Compute(3);
  var T2: task := go Compute(4);
  WriteLn(Wait(T1) + Wait(T2))
end.
```

The Mandelbrot showcase project in `examples/math/mandelbrot/` demonstrates this pattern: one task per row, all collected in order via `Wait`, combined with a live terminal UI.

## Standard Library

### Std.Task

Per-symbol reference (parameters, edge cases, `Wait` vs `WaitAll`): [std/task.md](std/task.md).

| Function | Signature | Description |
|----------|-----------|-------------|
| `Wait` | `(Handle: task): T` | Wait for a task and return its result |
| `WaitAll` | `(Tasks: array of task)` | Wait for all tasks to complete |

Here, `T` means the return type of the spawned call.

## Keywords

`go` — case-insensitive.

