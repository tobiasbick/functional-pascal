# Source-debugger roadmap

The source debugger includes detached controlled calls, read-only expression
evaluation, watches, conditional breakpoints, exact-hit conditions, and
non-stopping logpoints. Its current user and protocol documentation lives under
[`docs/pascal/tools/`](../../pascal/tools/debugger.md).

[deferred.md](deferred.md) lists all consciously postponed debugger work
together with its rationale and re-entry gates. Variable mutation is the next
remaining item; controlled calls do not imply DAP `setVariable` support.

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
