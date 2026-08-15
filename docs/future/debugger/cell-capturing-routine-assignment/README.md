# Cell-capturing routine assignment

Status: **planned; implementation has not started**  
Parent backlog item: [`DBG-D03`](../deferred.md)  
Last updated: 2026-08-15

## Objective

Extend the existing stopped-state function assignment path so a named nested
routine may be materialized with exact `Cell` and `EnclosingCell` captures from
its selected live lexical-owner frame. The constructed function remains owned
by that stopped task, shares the original cells, and is committed through the
existing atomic mutation path.

This is one runtime/tooling slice. It does not add FPAS syntax or change the
language rule that mutable-capture functions are task-bound.

## Fixed boundary

- Source: one uniquely resolved named nested routine already represented in
  verified executable metadata.
- Captures: existing `Value`, `Cell`, and `EnclosingCell` slots in recorded ABI
  order and with exact owner-binding identities.
- Owner: the exact selected lexical-owner frame in the current stop generation.
- Destination: a source-declared mutable function-typed frame register in that
  same frame and task; globals, cell-backed targets, and descendants are out.
- Effect: clone existing cell handles, construct one task-owned function value,
  validate it completely, and perform one existing atomic root commit.
- Surfaces: standard JSONL `variable.set` / `expression.set`, DAP `setVariable`
  / `setExpression`, and their existing VS Code Variables and Watch UI.

See [scope-and-decisions.md](scope-and-decisions.md) for the normative planning
boundary and [consciously-deferred.md](consciously-deferred.md) for exclusions.

## Document map

| File | Purpose |
|---|---|
| [scope-and-decisions.md](scope-and-decisions.md) | Stable decisions and non-negotiable limits |
| [architecture.md](architecture.md) | Runtime identity, task ownership, capture read, and commit design |
| [implementation-plan.md](implementation-plan.md) | Ordered work packages with stable `CCRA-*` IDs |
| [verification-matrix.md](verification-matrix.md) | Acceptance cases, evidence owners, and status |
| [progress.md](progress.md) | Checkpoint, evidence log, and exact resume procedure |
| [consciously-deferred.md](consciously-deferred.md) | Remaining `DBG-D03` work and unrelated backlog |

## Start and resume

1. Read this file, [scope-and-decisions.md](scope-and-decisions.md), and the
   current checkpoint in [progress.md](progress.md).
2. Recheck `git status --short`, the current branch, and all paths listed in
   [implementation-plan.md](implementation-plan.md); the working tree may have
   changed since this plan was written.
3. Establish the baseline commands in `CCRA-01` before editing runtime code.
4. Implement work packages only in dependency order and update both
   [progress.md](progress.md) and [verification-matrix.md](verification-matrix.md)
   after each exit gate.
5. Stop on any stop rule instead of widening the package silently.

The package remains under `docs/future/` until all matrix rows are supported by
recorded evidence. User-facing documentation must describe the capability only
after it exists. Delete this plan only after an explicit cleanup request.
