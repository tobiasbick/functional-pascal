# Std.Errors

Shared native causes preserve the attempted hosted operation and its original
explanation. Import the unit with `uses Std.Errors as Errors;`.

`HostError.Failure(Operation: string; Resources: array of string;
Domain: Option of string; Code: Option of integer; Detail: string)` is a native
error with the fixed label `Host operation failed`.

Operation names the qualified attempted operation. Resources retain supplied
identifiers in parameter order, such as source then target; operations without
resources use an empty array. Constructing or displaying an error does not
resolve paths, inspect resources or recover additional host data.

Domain and Code are both present for a raw OS code (`errno` on Unix, `win32` on
Windows), and both absent when the backend has no raw OS code. Native numeric
codes are not portable on their own. Backends without OS codes and host deadline
expiry retain their actual operation/resources with both fields absent. No
native code is invented or assigned to the wrong code space.

Detail retains the diagnostic explanation. Branch on the surrounding domain
variant rather than interpreting this text. Known native categories select a
typed case while preserving this original leaf; unclassified I/O failures use
`IoFailure` where the owning family provides it. Input validation has no
fabricated host cause. Runtime panics, task failures and compiler/VM invariants
retain their runtime diagnostic semantics.

Filesystem, process, secure-random, network and server families use this leaf
where their actual hosted operation failed. Their error families remain distinct
nominal types. HTTP transport errors retain `NetError`, including its nested
native cause; URI, UTF-8 and protocol validation use their own domain cases.
Use `Message()` only at an output or explicitly text-returning application
boundary. Existing application/SDK/TUI string Result contracts remain valid
through explicit mappings at those boundaries.

See [native error families](../../language/types/errors.md) for language rules
and typed causes, and the owning [Std unit pages](../README.md) for exact cases,
success values, EOF, cancellation and timeout behavior.
