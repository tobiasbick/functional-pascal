# Source-debugger roadmap

The source debugger includes detached controlled calls, read-only expression
evaluation, watches, conditional breakpoints, exact-hit conditions,
non-stopping logpoints, and stopped-state mutation of supported mutable values.
Its current user and protocol documentation lives under
[`docs/pascal/tools/`](../../pascal/tools/debugger.md).

[deferred.md](deferred.md) lists all consciously postponed debugger work
together with its rationale and re-entry gates. Task and concurrent debugging
is the next remaining implementation-sized slice; no implementation plan has
been selected for it yet.

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
