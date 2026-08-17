# UMB-80 scope and decisions

## Shared invariants

1. Runtime identities never depend on display text or adapter-local handles.
2. A rejection changes no worker, scheduler result, waiter, stop generation,
   or adapter state.
3. JSONL and DAP call the same session or host operation. VS Code maps to
   that adapter behavior.
4. No FPAS syntax, semantics, or language documentation changes are in scope.
5. Recordings observe all-stop quiescence from `UMB-40A` and debuggee-channel
   separation from `UMB-50`.
6. Memory, disk, event count, snapshot count, and retention stay bounded.
7. Forward execution with recording disabled stays unchanged.

## Current ownership inventory

These are inventory facts for `U80-00`, not acceptance of later children.

- Debug sessions are launch-owned and all-stop. JSONL advertises
  `reverse_execution: false`. DAP advertises `supportsStepBack: false`.
- No recording envelope, event log, or replay driver exists in `fpas-vm` or
  `fpas-debug`.
- Scheduler choices in a debug session run on one host execution lane.
  Hosted `Read`/`ReadLn` consume queued debuggee input. TUI and graph
  handlers run only as bytecode after resume.
- Durable watchpoint identities are executable globals from `UMB-70`.
  Inspection handles expire on resume.
- Attach, remote, and native debugging were rejected by `UMB-60`.

## `UMB-80A` — recording envelope and program identity

- Begins only after `U80-00` records current scheduler, host, and identity
  ownership.
- The envelope must name source and program identity without host paths.

## `UMB-80B` — scheduler and host-event capture

- Begins only after the envelope can name a recording.
- Capture is allowed only at explicit all-stop or host-input boundaries.

## `UMB-80C` — bounds and retention

- Begins only after capture points exist.
- Default paths must not retain unbounded events, snapshots, or files.

## `UMB-80D` — unsupported effects and recording-off

- Unsupported nondeterminism is rejected before a recording claims
  replayability.
- The recording-disabled path must match current launch-owned execution.

## Out of scope

- Attach/remote (rejected by `UMB-60`).
- Hot reload (`UMB-90`).
- In-call host interruption (rejected by `UMB-50D`).
- Non-stop execution (rejected by `UMB-40D`).
- Unauthenticated exposure of recordings, sources, or host metadata.
