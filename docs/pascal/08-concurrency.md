# 8. Concurrency

Functional Pascal provides Go-inspired concurrency with lightweight tasks, typed channels, and a `select` statement for multiplexing. Tasks created with `go` execute in parallel across multiple OS threads automatically.

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

## Channels

Channels are typed conduits for communication between tasks. The type `channel of T` is part of the language. Import `Std.Channel` for the channel operations shown below.

### Creating Channels

```pascal
uses Std.Channel;

var Ch: channel of integer := Make();
```

Create a buffered channel with a specific capacity:

```pascal
var Ch: channel of string := MakeBuffered(10);
```

### Sending and Receiving

```pascal
Send(Ch, 42);
var Value: integer := Receive(Ch);
```

From the program's point of view, `Send` blocks when the buffer is full and `Receive` blocks until a value is available. Internally the runtime retries cooperatively until the operation can proceed.

### Non-Blocking Receive

`TryReceive` returns an `Option` — `Some` with the value if one is available, `None` otherwise:

```pascal
uses Std.Channel, Std.Option;

var V: Option of integer := TryReceive(Ch);
if IsNone(V) then
  WriteLn('no value yet')
```

### Closing Channels

```pascal
Close(Ch);
```

After closing, pending values can still be received. Sending to a closed channel causes a runtime error.

### Example: Producer/Consumer

```pascal
program ProducerConsumer;
uses Std.Console, Std.Channel, Std.Task;

procedure Producer(Ch: channel of integer);
begin
  Send(Ch, 99)
end;

begin
  var Ch: channel of integer := Make();
  go Producer(Ch);
  WriteLn(Receive(Ch))
end.
```

## Select

The `select` statement waits on multiple channel operations simultaneously:

```pascal
select
  case Msg: string from Ch1:
    WriteLn('Got message: ' + Msg);
  case Num: integer from Ch2:
    WriteLn('Got number: ' + IntToStr(Num));
end
```

Each arm tries a non-blocking receive from its channel. The first arm with an available value executes. If no arm is ready and no `default` arm is present, `select` behaves like a blocking wait: the runtime keeps yielding until one arm becomes available. A `select` must contain at least one `case` arm or a `default` arm.

### Select with Default

A `default` arm runs when no channel has data. A `select` may also consist of a `default` arm only:

```pascal
select
  case Msg: string from Ch:
    WriteLn(Msg);
  default:
    WriteLn('No message available');
end
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

## Standard Library

### Std.Channel

| Function | Signature | Description |
|----------|-----------|-------------|
| `Make` | `(): channel of T` | Create a channel (buffer capacity 1) |
| `MakeBuffered` | `(Size: integer): channel of T` | Create a buffered channel |
| `Send` | `(Ch: channel of T; Value: T)` | Send a value (blocks when full) |
| `Receive` | `(Ch: channel of T): T` | Receive a value (blocks when empty) |
| `TryReceive` | `(Ch: channel of T): Option of T` | Non-blocking receive |
| `Close` | `(Ch: channel of T)` | Close the channel |

### Std.Task

| Function | Signature | Description |
|----------|-----------|-------------|
| `Wait` | `(Handle: task): T` | Wait for a task and return its result |
| `WaitAll` | `(Tasks: array of task)` | Wait for all tasks to complete |

Here, `T` means the return type of the spawned call.

## Keywords

`go`, `channel`, `select`, `default`, `from` — all case-insensitive.
