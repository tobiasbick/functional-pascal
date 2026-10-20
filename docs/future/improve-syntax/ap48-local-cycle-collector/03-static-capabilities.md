# AP48.3: Static capability diagnostics

Package: [AP48](README.md). Status: not started. Effort: medium.
Prerequisite: [AP48.2](02-transfer-rules.md) complete.

## Work

Represent known transferable, known task-bound and unknown callable-containing
values in the checker. Join control-flow states conservatively. Preserve the runtime
guard for unknown cases instead of introducing a new language effects system.

Track field/index writes, aggregate construction and updates, pattern bindings,
nested captures and method receivers. Ordinary function results and parameters
without a proven capability must remain unknown, rather than silently transferable.
Use available resolved metadata to improve known cases without assuming a complete
interprocedural analysis. Retain deferred checks for recursive named captures.

Diagnose known invalid starts, arguments, sends and task results at useful source
locations. Unknown values may proceed to the mandatory runtime check. Retain existing
diagnostic families where their meaning fits; document any necessary new diagnostic.

## Owners and migration

Modify `crates/fpas-sema/src/check/expr/task_bound.rs`, task-call checking, assignment
tracking, declaration metadata, closure pending-capability checks and the task/channel
standard-library checks. Keep representation and joins centralized. Review semantic
metadata and cached-artifact consumers if the representation changes.

## Acceptance and verification

- A known local closure remains local through wrappers, patterns, field/index writes
  and branch joins. Overwriting one path does not erase another path's local state.
- Helper-returned closures and callable parameters with unknown capability cannot
  bypass the VM guard, while valid immutable-returning helpers remain usable.
- Recursive captures and bound receivers receive consistent diagnostics.
- Plain scalar values extracted from local aggregates remain transferable.
- Compiler-detected violations and dynamic runtime violations implement the same
  ownership rule; uncertainty alone does not reject otherwise valid programs.

Extend sema capability tests and source-level closure/concurrency regressions.
Cover reused artifacts if metadata changes. Update diagnostic and closure guidance
with delivery; confirm the compact reference. No new FPAS syntax is planned.
