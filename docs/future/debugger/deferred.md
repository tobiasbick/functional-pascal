# Deferred debugger backlog

This file lists only unresolved capability packages that are not duplicated by
an active implementation plan. Implemented behavior and its supported limits
belong in [`docs/pascal/tools/`](../../pascal/tools/debugger.md).

Calls with host I/O, nondeterminism, blocking, tasks, opaque resources, or
unresolved dynamic effects are rejected by the implemented evaluation safety
policy. They are not automatically promises for future implementation.

## Open packages

| ID | Capability package | Deferred boundary | Re-entry gate |
|---|---|---|---|
| DBG-U01 | Source-debugger completion umbrella | The former `DBG-D03` through `DBG-D09` packages are owned without duplication by [`umbrella/`](umbrella/README.md) | Execute its dependency, risk, and acceptance gates one bounded package at a time |

## Maintenance rules

- Keep one row per independently specifiable unresolved package.
- Completing a package removes its row; current limitations stay in user-facing
  documentation and regression tests.
- Splitting a package replaces its parent row. Do not append children while
  retaining the same parent description.
- Active plans use stable work IDs, evidence, resume instructions, and their
  own consciously-deferred file. While an umbrella is active, keep only its
  single owning row here. On completion, retain only genuinely independent
  unresolved packages.
- Do not use this backlog to preserve implementation history.

Resume the active umbrella through its `progress.md`. When it closes, replace
`DBG-U01` only with independently unresolved packages; do not restore completed
or rejected implementation approaches.
