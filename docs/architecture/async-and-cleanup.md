# Async lowering and cleanup

The compiler emits IAsyncStateMachine types and AsyncTaskMethodBuilder calls
directly. It uses .NET tasks and awaiters, without generated C#, Task.Run wrappers,
blocking await wrappers, or a language scheduler. The source contract is in
[concurrency](../pascal/language/concurrency/README.md).

## State machines

Async functions and lambdas return Task/Task<T>. Their body returns the inner
result; an async program has a synchronous CLR entry bridge waiting for its
program task. Ordinary async calls start on the caller and suspend only at an
incomplete await.

Typed resumable blocks retain locals, parameters, generic parameters, closure
storage, exception handlers, and pending exits across suspension. Await consumes
the supported Task/ValueTask families and their ConfigureAwait forms. Context
capture, cancellation, and faults retain ordinary .NET behavior.

Requires checks run inside the state machine before the user body. A successful
return retains its result, completes cleanup, then executes ensures on the inner
value before completing the task. Fault, cancellation, failed cleanup, or a
condition-evaluation exception bypasses the successful postcondition path.

## Protected regions and exits

Synchronous using acquires once before its protected region and disposes through
finally. Await using awaits reverse-order disposal before completing any scope
exit, including return, break, continue, fault, and cancellation. A failed later
acquisition still releases earlier acquisitions. Null resources skip disposal;
value-type resources use constrained calls without boxing away their storage.

Await is supported in catch and finally. Exception filters preserve CLR first-pass
ordering before unwinding; a throwing filter rejects that filter and preserves
the original exception. Throwing cleanup replaces the pending exception while
outer cleanup still runs. A retained return value is published only after cleanup
and postconditions succeed.

Synchronous and async loops use the same typed comparison, endpoint, and step
operations. Counting bounds evaluate once; checking the endpoint before stepping
prevents overflow or invalid subrange construction after the final iteration.
Repeat continues reach the until condition after cleanup, while break bypasses it.

## Managed-reference lifetime

FP3030 rejects unsupported async contexts/awaitables and a managed reference that
would remain live across suspension. Async declarations cannot have ref/out/in
parameters. A caller may await argument values before selecting writable storage;
it cannot retain a struct receiver address while awaiting a later handler or call
argument. Reference receivers and ordinary delegate values may survive suspension.

Async procedures/void, custom awaiters, awaits in filters, async enumeration, and
general ref-like values are outside the supported source contract. Rejected
metadata is diagnosed before emission.

## See also

- [Architecture overview](README.md)
- [Contracts](../pascal/language/functions/contracts.md)
- [Resource lifetime](../pascal/language/error-handling/resources.md)
- [Debug information](debug-information.md)
- [Async and resource tests](../../tests/coverage.md#language-and-metadata)
