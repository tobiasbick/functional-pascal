# Deferred source-debugger work

The V1 source debugger is implemented. Its current user and protocol
documentation lives under [`docs/pascal/tools/`](../../pascal/tools/debugger.md).

This directory retains only capabilities that are consciously postponed. See
[deferred.md](deferred.md) for their rationale, required safety properties, and
explicit re-entry gates. None of those entries describe current behavior.

The implemented debugger does not change FPAS syntax, semantics, or the
language specification.
