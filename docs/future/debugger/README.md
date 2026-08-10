# Source-debugger roadmap

The source debugger, including read-only expression evaluation, watches,
conditional breakpoints, exact-hit conditions, and non-stopping logpoints, is
implemented. Its current user and protocol documentation lives under
[`docs/pascal/tools/`](../../pascal/tools/debugger.md).

[deferred.md](deferred.md) retains the capabilities that remain consciously
postponed, together with their rationale, required safety properties, and
explicit re-entry gates.

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
