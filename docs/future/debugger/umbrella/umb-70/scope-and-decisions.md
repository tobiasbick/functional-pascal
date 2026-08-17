# UMB-70 scope and decisions

## Shared invariants

1. Runtime identities never depend on display text or adapter-local handles.
2. A rejection changes no worker, scheduler result, waiter, stop generation,
   or adapter state.
3. JSONL and DAP call the same session or host operation. VS Code maps to
   that adapter behavior.
4. No FPAS syntax, semantics, or language documentation changes are in scope.
5. Data stops and actions observe all-stop quiescence from `UMB-40A`.
6. Overhead and retained watchpoint state stay bounded.

## Current ownership inventory

These are inventory facts for `U70-00`, not acceptance of later children.

- Source and function breakpoints live in `fpas-vm` debug breakpoints. Runtime
  failure filters and hit/log policies live in `fpas-debug`.
- Stopped-state mutation already validates some portable identities. Capture-cell
  destinations remain rejected until this package proves cell identity and
  lifetime (`U10D-CELL` → `UMB-70A`).
- DAP does not advertise data breakpoints. JSONL has no data-breakpoint
  command.
- Attach and remote debugging were rejected by `UMB-60`. Native OS debugging
  was rejected by `UMB-60C`.

## `UMB-70A` — stable observable data identities

- Begins only after `U70-00` records current mutation and breakpoint ownership.
- Globals, frame registers, cells, and supported descendants need exact
  lifetimes before watchpoints can name them.

## `UMB-70B` — data breakpoints

- Begins only after `UMB-70A` can name locations without display parsing.
- Read/write/change semantics and bounded overhead must be deterministic
  across tasks.

## `UMB-70C` — mutating breakpoint actions

- Begins only after data stops exist. Reuse prepare/validate/commit.
- Mutating actions invalidate snapshots exactly once.

## Out of scope

- Attach/remote (rejected by `UMB-60`).
- Record/replay (`UMB-80`) and hot reload (`UMB-90`).
- In-call host interruption (rejected by `UMB-50D`).
- Non-stop execution (rejected by `UMB-40D`).
