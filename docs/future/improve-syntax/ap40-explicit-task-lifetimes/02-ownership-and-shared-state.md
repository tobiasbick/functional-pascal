# AP40.2: Ownership and shared state

Package: [AP40](README.md). Status: blocked by AP40.1 and AP30.2.

## Scope and affected areas

Implement the selected ownership boundary and shared-state rules. Inspect
`crates/fpas-sema/src/check/task_scopes/`, `check/discard/`, closure/capture
analysis, `crates/fpas-vm/src/vm/worker.rs`, `globals.rs`, and task scheduling.
Share native/user mutation restrictions with AP37's rule when applicable;
do not silently allow writable references to cross task boundaries.

## Migration and documentation

Migrate helper spawns and shared mutable state using the selected explicit
ownership or channel-based patterns. Preserve intended completion and failure
handling. Update concurrency/go/scope/capture pages, Std.Tasks, discard rules,
grammar if needed, compact reference, diagnostics, and editor tools.

## Verification

Test inline versus extracted starts, named/anonymous helpers, scopes/groups,
normal and early exits, cancellation, fallible results, handle escapes, and
any permitted discard. Exercise local and global shared-state rules through
source-level checks and deterministic runtime synchronization tests rather
than relying on a flaky lost-update race. Run required implementation checks.
