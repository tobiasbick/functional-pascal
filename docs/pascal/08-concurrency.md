# 8. Concurrency

Functional Pascal provides Go-inspired lightweight task concurrency. Tasks created with `go` may run on worker threads in parallel with the main program; the main program always runs on the OS thread that starts execution.

## Bytecode mapping

The compiler lowers `go` to dedicated VM opcodes:

- **`go` as an expression** (e.g. assigned to a `task` variable) emits a **retained** spawn: the callee and arguments are popped and a task handle is pushed for later `Wait`.
- **`go` as a statement** (fire-and-forget) emits a **detached** spawn: same stack effect except **no** handle is retained for the caller.

At startup, the runtime scans the compiled instruction stream: if the program contains **no** retained or detached spawn opcodes, it does **not** start background worker threads. Opcodes used only for cooperative scheduling (for example **`Yield`**) do **not** by themselves imply a pool — only spawn opcodes do.

## Tasks

Launch a concurrent task with the `go` keyword.

### Expression form (handle retained)

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

### Statement form (fire-and-forget)

A `go` **statement** runs the call concurrently and **does not** produce a handle (the compiler discards the task result at the bytecode level). Use this when you only need side effects:

```pascal
go LogEvent('started');
```

### What `go` may target

`go` must be followed by a **single call expression** (not a bare designator or arbitrary value). The callee may be:

- a **function** or **procedure** (including qualified names such as `Std.Console.WriteLn(...)`),
- a **method** call, or
- a call through a **callable variable** (function type, procedure type, and similar).

Bare values, operators, and non-call expressions are rejected by the parser or semantic checker.

### Thread pool

If the compiled program contains **no** spawn opcodes for tasks (equivalently: the program never uses `go` in a way that reaches bytecode), the runtime does **not** start background worker threads.

If it does emit spawn bytecode, the runtime starts **`max(1, available_parallelism − 1)`** worker threads that share a ready queue, while the **main task** (task id `0`) still runs on the thread that started execution. Each pool thread runs **at most one** ready task at a time: workers block when the queue is empty and are woken when work is enqueued or the runtime shuts down. Together, this matches typical machine parallelism without starting idle workers for programs that never spawn tasks.

Background workers exist only for a single program run: the runtime **joins** pool threads before execution returns so short-lived hosts do not accumulate stray threads across many runs.

When the **main task** finishes (normally or with a runtime error), the runtime signals **teardown shutdown** so idle workers wake and exit once the ready queue is drained. This is separate from **task failure**: when a spawned task aborts with a runtime error, other spawned work may be stopped cooperatively at the next instruction boundary. The host surfaces **one** primary diagnostic: if the main task failed, that error wins; otherwise a worker error (for example after a spawned task **`panic`s**) is reported.

**Cooperative scheduling:** Spawned tasks can be **preempted cooperatively** after a fixed instruction budget and on the **`Yield`** opcode so long-running bytecode cannot starve other queued tasks on the same worker. The **main** program task always runs on the thread that started execution and is **not** placed on the shared ready queue; a main-thread `Yield` yields the OS thread so pool workers can run.

### Shared runtime state

Worker threads and the main execution thread share one runtime state: immutable bytecode, a mutex-protected **ready queue** of suspended tasks paired with a **condition variable** so idle workers block instead of spinning, **task id** allocation, **task result** storage for handles used with `Wait`, a **shutdown** flag, and mutex-protected **console**, **input**, and **TUI** state so concurrent tasks do not corrupt I/O. Hosted TUI `On*` handlers run on the **main** thread only.

### Task Type

The `task` type represents a handle to a running task. Assign the result of a **`go` expression** to capture it. For type checking, the handle carries the spawned call’s result type **`T`** (for a procedure spawn, **`T`** is the empty / unit result); at runtime the value is an opaque task id.

```pascal
var T: task := go ComputeSomething(Data);
```

## Task Management

### Waiting for a Task

`Std.Task.Wait` blocks until the task completes and returns its result type **`T`** (the runtime waits on the same shared condition variable as the task queue — it does not hot-spin):

```pascal
var T: task := go Compute(100);
var Result: integer := Wait(T);
```

For a **procedure** task, `Wait` completes when the procedure finishes; **`T`** is the unit type in the type system.

### Waiting for Multiple Tasks

`Std.Task.WaitAll` blocks until all tasks in the array complete (same condvar-based blocking as `Wait`):

```pascal
WaitAll([T1, T2, T3]);
```

`WaitAll` is a barrier only: it does not consume return values. You may still `Wait` each handle afterward. See [std/task.md](std/task.md).

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

Per-symbol reference (parameters, edge cases, `Wait` vs `WaitAll`, runtime errors): [std/task.md](std/task.md).

| Function | Signature | Description |
|----------|-----------|-------------|
| `Wait` | `(Handle: task): T` | Wait for a task and return its result |
| `WaitAll` | `(Tasks: array of task)` | Wait for all tasks to complete |

Here, **`T`** is the return type of the spawned call (unit for a procedure).

## Keywords

`go` — case-insensitive.
