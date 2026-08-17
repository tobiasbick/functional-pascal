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

These are inventory facts for `U70-00`/`U70-01`, not acceptance of later children.

- Source and function breakpoints live in `fpas-vm` debug breakpoints. Runtime
  failure filters and hit/log policies live in `fpas-debug`.
- Stopped-state mutation already validates some portable identities. Capture-cell
  destinations remain rejected until this package proves cell identity and
  lifetime (`U10D-CELL` → `UMB-70A`).
- JSONL advertises `data_breakpoints: true` with `data_breakpoint_access`
  `write` and `change`, `location_describe: true`, and `breakpoint_assign:
  true`. DAP advertises `supportsDataBreakpoints: true` and maps
  `dataBreakpointInfo` / `setDataBreakpoints`. Optional `assign` forwards on
  `setBreakpoints` and `setDataBreakpoints`. Inspection handles expire on
  resume and are not watchpoint identities; global location identities are.
- Attach and remote debugging were rejected by `UMB-60`. Native OS debugging
  was rejected by `UMB-60C`.

## Frozen identity inventory (`U70-01`)

Stopped mutation uses `MutationTarget`: a `MutationRoot`, descendant path,
expected type, `inspection_generation`, and optional `frame_id`. Roots are
`FrameRegister`, `Global`, and `ClosureCell`. Those handles are stop-scoped.
`variables_reference` and `frame_id` expire on resume and cannot name a
watchpoint that must survive continue.

- Globals have executable-stable slot indexes, but mutation still binds them to
  the current stop generation.
- Frame registers exist only while that frame is live in the current snapshot.
- `ClosureCell` retains an `Arc<Mutex<Value>>` for stopped writes. It is not an
  owner-task or alias registry identity. Task-bound function assignment still
  rejects capture-cell destinations (`U10D-CELL`).
- Supported descendants (record fields, array indexes, dictionary values, enum
  payload fields, Result/Option wrappers) inherit the root lifetime.

Do not add further mutating-action modules until a later package. Capture-cell
destinations stay rejected: `ClosureCell` has pointer identity but no owner-task
or alias registry (`unregistered_alias`).

## Proven identity subset (`U70-10`)

- Globals: executable-stable slot index. Lifetime `executable`. Survives
  continue for the session.
- Frame registers: `(task_id, function, register)` while that activation is on
  the stack. Lifetime `live_frame`. The protocol handle still expires on
  resume; re-describe from a fresh stop if the frame is still live.
- Capture cells: no issued identity. Lifetime `unregistered_alias`. Task-bound
  function assignment keeps rejecting capture-cell destinations.
- Descendants inherit the root kind and lifetime.

JSONL `location.describe` and DAP `fpas/locationDescribe` expose that subset.

## Proven data-breakpoint subset (`U70-20`)

- Watch executable globals only. Frame-register identities stay unverified.
  Capture cells have no identity and remain unwatchable.
- Access `write` (any debug-owned store to that global index) and `change`
  (that store and the new value differs from the snapshot taken at resume).
  `read` / `readWrite` stay unverified. Loads are not instrumented.
- Descendants of a global watch the root slot, including index-path stores.
- Logical data breakpoints share the existing 256 breakpoint limit with source
  and function breakpoints. Replace-all is atomic.
- No condition, hit, or log policy on data breakpoints unless `assign` is
  present. Missing policy still stops. Optional `assign` uses the shared
  source-breakpoint policy after the watch hits.
- JSONL `data_breakpoints.replace` and DAP `dataBreakpointInfo` /
  `setDataBreakpoints` map the same engine. `data_breakpoint.set` stays
  rejected. VS Code uses the standard variable data-breakpoint UI.

## Proven mutating-action subset (`U70-30`)

- Optional `assign: { identity, expression }` on source `breakpoint.set` and
  data `data_breakpoints.replace` items.
- Identity must be an executable global. Frame registers and capture cells are
  rejected at set time without creating the breakpoint.
- Policy order is condition → hit → assign → log-or-stop.
- Assign reuses prepare/validate/commit. Snapshots invalidate once per
  successful assign. Failures leave storage and generation unchanged.
- After a successful assign, log interpolation re-reads `frame_id` from the
  refreshed stack.
- Function breakpoints reject `assign`, `action`, and `logMessage`.
- JSONL advertises `breakpoint_assign: true`. DAP forwards `assign`; VS Code
  has no extra assign command.

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
