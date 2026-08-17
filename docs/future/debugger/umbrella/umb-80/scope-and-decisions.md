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
  `reverse_execution: false` and `record_replay: false`. DAP advertises
  `supportsStepBack: false`.
- No recording envelope, event log, or replay driver exists in `fpas-vm` or
  `fpas-debug`.
- Scheduler choices in a debug session run on one host execution lane.
  Hosted `Read`/`ReadLn` consume queued debuggee input. TUI and graph
  handlers run only as bytecode after resume.
- Durable watchpoint identities are executable globals from `UMB-70`.
  Inspection handles expire on resume.
- Attach, remote, and native debugging were rejected by `UMB-60`.

## Frozen recording-off contract (`U80-01`)

Recording and reverse execution stay off until later children capture or
reject each named effect. A named reject changes no stack, frame, task, or
adapter state. Missing effects below are tests or recorded bounds, not a
replayability claim.

Observable today on the recording-off path:

- All-stop instruction-boundary dispatch on one debug host lane; task IDs
  stay stable for the session.
- Structured console output events.
- Queued debuggee `Read`/`ReadLn` (`io.input`).
- Durable global identities, data breakpoints, and breakpoint assign from
  `UMB-70`.

Not replayable until captured or rejected:

- `Std.Random`
- Wall-clock and `Std.Time` beyond debug Sleep (`DebugClock::realtime()` on
  launch-owned sessions; `manual()` exists only in VM tests)
- `Std.Fs`, environment, process, network, and path host effects
- OS TUI `ReadKey`/`ReadEvent` and graph `On*` handlers (they run only as
  bytecode after resume and are not recorded)
- Non-debug worker-pool scheduling (`fpas run`)
- Host paths in any future envelope (privacy)

Named rejects without resume or mutation:

- JSONL `step_back`, `reverse_continue`, and `replay`
- DAP `stepBack` and `reverseContinue`

JSONL `record` and DAP `fpas/record` start capture after `U80-20`; they still
must not resume.

## `UMB-80A` — recording envelope and program identity

- Begins only after `U80-01` freezes recording-off and named rejects.
- The envelope names versioned program identity and portable sources.
- Host filesystem paths are rejected without echoing the path or mutating
  the session. JSONL `recording.describe` and DAP `fpas/recordingDescribe`
  expose that identity. Replay and reverse-step remain rejected.

## `UMB-80B` — scheduler and host-event capture

- Begins only after the envelope can name a recording.
- Capture is off until JSONL `record` or DAP `fpas/record`.
- Events are recorded only at explicit all-stop boundaries and successful
  queued `Read`/`ReadLn` input. Mid-resume scheduler choices are not logged.
- A second `record` is idempotent. Capture does not resume or enable replay.
- An in-memory ceiling of 4096 events is enforced; later events are dropped
  and `truncated` is reported.

## `UMB-80C` — bounds and retention

- Begins only after capture points exist.
- Default paths must not retain unbounded events, snapshots, or files.
- Capture keeps at most 4,096 events in session memory. Overflow drops later
  events, leaves forward execution unchanged, and reports `truncated`.
- No recording files are written (`recording_disk` is false).
- No recording snapshots are retained (`recording_snapshots` is 0).
- Retention is the debug session lifetime.

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
