# Phase 6 — Production-readiness gate

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).

## Task 6.1 — Inventory production dependencies

**Status:** complete.

Produce a checked-in table listing every application, example, test, document, manifest, compiler/
VM module, and workflow that references `Std.Tui`, `Std.Tui2`, Turbo Vision intrinsics, or their
paths. Include exact `rg` commands and classify each row as port, delete, replace, or intentionally
retain until promotion. This is an inventory task; it must not delete or port files.

Completed in [production-inventory.md](../production-inventory.md). The IDE is retired legacy
source, excluded from Tui3 builds and tests; `Std.Tui2` has no application consumer.

## Gate 6.A — Select the representative production flow

**Status:** not applicable.

**Prerequisite:** Task 6.1.

There is no representative production application. `apps/ide/` is retired legacy source and is
explicitly excluded from Tui3 builds and tests.

## Task 6.2A — Plan the approved flow and audit gaps

**Status:** not applicable.

No production flow is in scope.

## Gate 6.B — Decide feature gaps and effects

**Status:** not applicable.

No production feature gaps are in scope.

## Task 6.2B — Port the approved flow

**Status:** not applicable.

No production flow is in scope.

## Task 6.3 — Repeat performance and completeness evidence

**Status:** complete.

Repeat the Phase 0 measurements with the complete Tui3 control set. Record tree sizes, terminal
sizes, iterations, timings, and clone/allocation evidence in [testing.md](../testing.md). Confirm
Tui3 docs/tests cover every public symbol that will be renamed and rerun the full checkpoint. The
retired IDE is not a checkpoint input.

Completed evidence is recorded in [testing.md](../testing.md): repeated-frame dimensions and
timing, full Tui3 suite, terminal rollback tests, workspace checkpoint, and formatting checks.

## Gate 6.C — Promote decision

**Status:** complete.

**Prerequisite:** Task 6.3.

The retired `Std.Tui` and `Std.Tui2` implementations are approved for removal. Phase 7 begins by
freezing the exact destructive manifest; the excluded legacy IDE is not a promotion prerequisite.
