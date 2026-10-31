# AP55 resource contract

This is the approved contract implemented by [AP55.2](02-implementation.md).

## Declaration and release

`using const Name[: Type] := Expression;` is a read-only local binding. Evaluate
the initializer once and register cleanup only after it succeeds. The compiler
protects the acquisition-to-registration handoff against cancellation. The
nearest lexical block owns the resource and releases successfully registered
resources in reverse order whenever that block exits. This applies to implicit
control bodies as well as explicit blocks, programs and routines, including
generic, imported and anonymous code.

A custom record has at most one `release procedure Close(Self: Resource);`
member. `release` is contextual. Ordinary visibility applies to explicit source
calls; automatic release is a capability of the record even when its release
member is private. Unit interfaces retain that capability for importing code.
The member has only `Self`, is neither static nor generic, and returns no value. A custom
release procedure must be idempotent if code invokes it explicitly because the
automatic invocation still occurs. Explicit native close retains its existing
contract; automatic release skips a native handle that is already closed.

The compiler catalogue selects native close operations for file and socket
handles, processes, ACP `Release`, MCP `Release` and `CloseEndpoint`, and HTTP
`CloseStream`. No public namespace allocation is needed. Exact supported handle
types and close mappings belong to the implementation catalogue.

Safe synchronous borrowing is allowed. Reject compile-known resource escape via
aliases, captures, storage, channels or return values using existing escape
inference. This does not introduce a general ownership type system.
Aliases that existed before the `using` binding remain valid and observe the
same native handle state after release.

## Exit and failures

Normal completion, return, `try` propagation, loop control, cancellation and
managed failure all unwind lexical obligations. Join task scopes and release
resources in reverse lexical nesting order; routine postconditions run after
cleanup. Mask cancellation during cleanup, including suspended native work.
Native release joins physical cleanup and retires the resource before the
enclosing block completes. An already closed native handle is skipped.

A pending Result error or None remains the result when cleanup fails; report
cleanup failure through a supplemental diagnostic. A cleanup failure replaces
success or cancellation. An existing panic remains primary, as does the first
non-cancellation runtime failure; further cleanup failures are reported separately.
Continue remaining cleanup even after a release failure.
If a release procedure's nested resource fails to release, the enclosing
procedure does not resume after that failure; outer obligations still run.

Debugger forced return or restart must not abandon active obligations. Preserve
normal stepping and cancellation. `finally`, `defer`, exception catching and
general destructors are outside this contract.
