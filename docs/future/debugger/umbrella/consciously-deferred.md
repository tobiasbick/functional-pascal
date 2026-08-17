# Umbrella boundaries

This file records what the umbrella does not authorize. It does not duplicate
the capability packages owned by [implementation-plan.md](implementation-plan.md).

## Outside the umbrella

- FPAS syntax, static semantics, runtime language semantics, or language-spec
  changes without separate explicit user agreement.
- A second debugger engine in JSONL, DAP, VS Code, a remote agent, or a native
  adapter.
- Marketplace/Open VSX publication, telemetry collection, cloud hosting, or
  repository CI automation unless separately requested.
- Unauthenticated remote control or default exposure of host paths, environment
  data, sources, recordings, or terminal contents.
- Unbounded history, recordings, retained completed tasks, output, snapshots,
  or breakpoint actions.
- Unsafe thread termination as an implementation of pause or cancellation.
- Backward-compatibility layers for obsolete internal debugger protocols unless
  a concrete supported consumer is identified.

## Decisions, not promises

The following boundaries remain inside the umbrella as explicit decisions or
prerequisites, not promises that implementation will be accepted:

- Dynamic callable endpoints, opaque identity-bearing resources, and in-place
  callable editing were rejected by `UMB-10D`; its evidence remains in parent
  progress, focused tests, and current debugger documentation. Task-bound
  capture-cell destinations remain blocked on `UMB-70A` cell identity and
  lifetime.
- Arbitrary instruction changes were rejected by `UMB-30D` / `U30-50`. Existing
  bytecode verification proves the original CFG from entry, not initialized
  registers or operand types at an interior sequence point. Temporary registers
  are reused. Function-entry reconstruction remains `frame.restart`. Evidence
  remains in parent progress, focused tests, and current debugger documentation.
- Non-stop execution, scheduler shortcuts, and persistent history in
  `UMB-40D`.
- In-call interruption of blocking host intrinsics in `UMB-50D`. Pause remains
  cooperative after the intrinsic returns; empty-queue `ReadLn` fails with
  `F4011` rather than waiting. Unsafe thread kill stays forbidden.
- OS-level native debugging in `UMB-60C`. The debuggee is FPAS bytecode in
  `fpas-vm`; gdb/lldb of the host process would be a second semantic engine.
  Disassemble and memory requests stay unsupported.
- Record/replay and hot reload in `UMB-80` and `UMB-90`.

A rejected decision must state the missing invariant or disproportionate cost.
Only a remaining independently useful capability is returned to
[`../deferred.md`](../deferred.md); rejected implementation approaches are not
preserved as backlog.

## Maintenance

- Do not copy umbrella child lists into `../deferred.md` while this plan is
  active.
- Do not describe unimplemented behavior under `docs/pascal/`.
- Remove obsolete exclusions when their capability passes acceptance.
- Delete this file with the umbrella after `UMB-99`.
