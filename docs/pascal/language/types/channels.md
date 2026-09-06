# Channel types

`channel of T` is a built-in handle type for bounded FIFO communication between tasks. The element
type is part of the static type: `channel of integer` and `channel of string` are different types.

```pascal
var Messages: channel of string := Std.Task.CreateChannel(16)
```

## Type rules

- The syntax is `channel of type`, and nested type expressions are allowed.
- A channel accepts only values compatible with its element type.
- Receiving from `channel of T` returns `result of T, string`.
- Channel handles may be copied and passed to spawned tasks. Copies refer to the same FIFO queue.
- A value whose closure state is task-bound cannot be sent through a channel.
- `CreateChannel` has no value argument from which to infer `T`, so its result must be used where a
  concrete channel type is declared, passed, or returned.

## Lifetime and closure

Channels belong to the VM that created them. `CloseChannel` is idempotent. Its first successful
close returns `true`; later calls return `false`.

Closing wakes blocked senders and receivers. Values buffered before the close remain available in
FIFO order. After the buffer is drained, receives return `Error('Channel is closed')`. Sends after
the close return the same error without accepting the value. VM shutdown closes every channel and
wakes blocked operations.

Channel capacity is fixed at creation and must be in `1..=1048576`. There is no unbounded channel
form.

Operations are provided by [`Std.Task`](../../std/concurrency/task.md).

## See also

- [Concurrency](../concurrency/README.md)
- [Result and Option types](result-option-types.md)
