# Asynchronous execution and concurrency

Async FPAS uses ordinary CLR tasks, cooperative cancellation, and shared-state
rules. The language does not add a separate task scheduler or ownership scope.

| Topic | Description |
| --- | --- |
| [Async functions and lambdas](async-functions.md) | Task results, return behavior, and contracts |
| [Await](await.md) | Task/ValueTask families, context capture, faults, and cancellation |
| [Async programs](async-programs.md) | Await in entry code and the generated host bridge |
| [Cancellation and coordination](cancellation-and-coordination.md) | Tokens, Task APIs, channels, and an HTTP example |
| [Asynchronous disposal](../error-handling/async-disposal.md) | Cleanup that completes before scope exit |

## See also

- [Functions](../functions/README.md)
- [Resources](../error-handling/README.md)
- [CLR interop](../../interop/README.md)
- [Async implementation](../../../implementation/overloads-interfaces-debugging-and-async.md)
