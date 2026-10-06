# Discarding values

`discard Expression;` evaluates an expression exactly once and explicitly
ignores its value. `discard` is a reserved, case-insensitive keyword.

```pascal
discard 42;
discard Some('unused');
discard Error('deliberately ignored');
```

A discarded `Result` or `Option` is not unwrapped. Expression side effects
and runtime failures still occur. Discarding a function or procedure value
does not invoke it. A procedure call produces no value and cannot be used
as the operand; call the procedure directly.

Function calls may also appear as statements. Explicit discard records the
intent to ignore their result.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`discard_stmt`).

## Task handles and aggregates

A task handle cannot be discarded. The compiler also rejects types with
task handles nested in arrays, dictionary keys or values, stored record fields,
enum payloads, `Option`, or either branch of `Result`. Named and recursive
types follow the same rules. Record method signatures are not stored fields.

The declared type determines this restriction: an empty array of task handles
or `None` of an option containing a task handle is still rejected. The compiler
checks every enum variant and both `Result` branches.

```pascal
discard go Worker(); // error: use the statement go Worker();
go Worker();         // fire-and-forget task
```

For an existing handle or a function returning a handle, retain and consume
the handle. Spawning a new task is not a replacement for handling that value.

## Generics

In a generic body, the declared constraints must exclude task handles for
every permitted type argument. `Numeric` and `Comparable` provide this
guarantee; an unconstrained `T` and `Printable` do not. The same check applies
inside aggregate types. Observed call sites do not relax the body check.

At a call site with a concrete result type, ordinary discard rules apply:
a generic call returning `integer` can be discarded. The check uses the
existing constraints and runs at compile time.

## Channels and callable captures

Channels recursively follow the rules for their element type. A
`channel of integer` can be discarded; a channel whose element type contains
task handles cannot. Queue contents and other references do not affect the
check. Channels with unknown callable captures in their element type are
also rejected.

A function or procedure value can be discarded only when its captures are
statically proven free of task handles. This includes captured containers,
channels, other closures, and the receiver of a bound record method. Merely
accepting or returning a task in the callable signature is not a stored
capture.

The compiler preserves known capture information through immutable bindings,
aggregate construction, routine results, and compiled-unit interfaces.
Record construction also checks captures in omitted fields' default values.
Callable parameters and values lacking capture information are rejected.
Mutable storage with callable contents is conservatively treated as having
unknown captures, since later assignments can change those contents.

Task-freedom is independent of the task-bound restriction on closures with
mutable state. Capturing a mutable `integer` does not introduce a task handle.

## See also

- [Runnable discard example](../../../../examples/pascal/functions/discard_values.fpas)
- [Capturing closures](closures.md)
- [Task handles](../concurrency/task-handles.md)
- [Channels](../types/channels.md)
- [Diagnostics](../../tools/diagnostics.md)
