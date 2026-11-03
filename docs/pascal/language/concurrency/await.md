# Await

`await Expression` is valid in an async function/lambda or an async program body.
It suspends without blocking the executing thread when completion is pending.
Awaiting `Task<T>` yields `T`; awaiting Task has no value and is a statement.
Awaiting a faulted/canceled operation propagates the corresponding .NET exception.

The baseline consumes Task, Task<T>, ValueTask, ValueTask<T>, and their standard
ConfigureAwait awaitables. It does not produce ValueTask from an async declaration
or support arbitrary custom awaiter patterns. ValueTask consumption must respect
its producer's .NET lifetime/reuse rules; it is not assumed reusable like Task.

Default continuation-context behavior matches ordinary .NET await. Call
`ConfigureAwait(false)` explicitly when appropriate. Await does not imply a new
synchronization context, thread affinity, or mutual exclusion.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
