# Phase 6 — Production-readiness gate

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).

## Task 6.1 — Inventory production dependencies

**Status:** blocked by Phase 5.

Produce a checked-in table listing every application, example, test, document, manifest, compiler/
VM module, and workflow that references `Std.Tui`, `Std.Tui2`, Turbo Vision intrinsics, or their
paths. Include exact `rg` commands and classify each row as port, delete, replace, or intentionally
retain until promotion. This is an inventory task; it must not delete or port files.

## Gate 6.A — Select the representative production flow

**Status:** human gate.

**Prerequisite:** Task 6.1.

A human names the concrete application and user flow to port and its required capabilities. “A
demo” or an agent-selected convenient subset is not sufficient.

## Task 6.2 — Port the approved flow

**Status:** blocked by Gate 6.A.

Create a file-level task list from the selected flow, then port it without compatibility adapters.
Record every unavailable capability in a feature-gap table. Do not silently reduce the approved
flow.

## Gate 6.B — Decide feature gaps and effects

**Status:** human gate.

**Prerequisite:** Task 6.2.

For each gap, explicitly choose implement, accept loss, or block promotion. Timers, worker results,
file dialogs, subscriptions, or other effects require a separate data-only transport design and an
FPAS feasibility spike before implementation. A cheaper implementation agent must not design that
transport opportunistically.

## Task 6.3 — Repeat performance and completeness evidence

**Status:** blocked by resolution of Gate 6.B.

Repeat the Phase 0 measurements with the complete control set and representative flow. Record tree
sizes, terminal sizes, iterations, timings, and clone/allocation evidence in
[testing.md](../testing.md). Confirm Tui3 docs/tests cover every public symbol that will be renamed
and rerun the full checkpoint.

## Gate 6.C — Promote decision

**Status:** human gate.

**Prerequisite:** Task 6.3.

Promotion requires an explicit recorded decision based on the inventory, approved flow, accepted
gaps, performance evidence, full tests, and terminal restoration evidence. A successful example
alone is insufficient.
