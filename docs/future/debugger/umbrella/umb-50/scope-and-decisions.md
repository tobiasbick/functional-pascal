# UMB-50 scope and decisions

## Shared invariants

1. JSONL and DAP protocol bytes must remain distinguishable from debuggee
   terminal, TUI, and graph I/O. A mixed unframed stream is forbidden.
2. Debuggee I/O is session-owned. Adapters map events; they must not own a
   second console, TUI, or graph runtime.
3. A rejection changes no worker, scheduler result, waiter, stop generation,
   or adapter state.
4. JSONL and DAP call the same session or host operation. VS Code maps to
   that adapter behavior.
5. No FPAS syntax, semantics, or language documentation changes are in scope.
6. Unsafe thread termination is forbidden as pause, cancellation, or host
   interruption.

## Current ownership inventory

These are inventory facts for `U50-00`, not acceptance of later children.

- `fpas debug` binds JSONL and DAP to process stdin/stdout in
  `crates/fpas-cli/src/cli_debug.rs`. JSONL live mode reads stdin lines;
  DAP uses `Content-Length` framing on the same stdio pair.
- JSONL `crates/fpas-debug/src/jsonl/transport.rs` keeps protocol stdout as
  JSON lines. Captured program output is re-emitted as structured `output`
  events, not as raw bytes on the protocol stream. DAP framing lives in
  `crates/fpas-debug/src/dap/framing.rs` and maps those events.
- `DebugSession` owns a debuggee channel that is connected at launch and
  closed by disconnect. `--output-limit` still bounds captured stdout.
  Stopped-state `io.input` / `fpas/input` queue hosted `Read`/`ReadLn` lines.
  Protocol stdin is never debuggee input.
- `Worker::execute_hosted_intrinsic` owns console, args, callbacks, graph,
  and test-host intrinsics. `Std.Console` Read/ReadLn/ReadKey block inside
  the intrinsic through `text_input` / `key_input`. Write/WriteLn mutate the
  hosted console and feed captured debugger output.
- Graph host callbacks live in `crates/fpas-vm/src/vm/hosted/graph/host.rs`.
  TUI cells and CRT-style console operations live in
  `crates/fpas-vm/src/vm/hosted/console.rs` (~407 lines; split before adding
  pause-in-host work).
- Pause requested during an in-progress host intrinsic remains cooperative:
  `cooperative_pause_waits_for_a_blocking_intrinsic_to_return` proves the
  pause is observed after the intrinsic returns. `UMB-50D` owns interruption
  inside that call.

## Frozen `UMB-50A` subset

- Protocol stdin is JSONL or DAP. Raw lines and unframed bytes are protocol
  errors; they are never hosted `Read`/`ReadLn` input.
- Protocol stdout is JSON lines or `Content-Length` frames. `WriteLn` appears
  only inside structured `output` events.
- The session owns a debuggee channel that is connected at launch and closed
  by disconnect without dispatching remaining bytecode.
- Protocol stdin is never hosted `Read`/`ReadLn` input. `UMB-50B` later
  queued those lines through `io.input` / `fpas/input`; `live_terminal`
  remains false.
- Protocol stdin EOF ends adapter `serve`; it is not debuggee stdin EOF.
- TUI/graph handlers run only as bytecode inside hosted intrinsics. All-stop
  inspection does not dispatch them. Debug hosted `KeyInput` never polls the
  process terminal.
- Pause requested during a blocking host intrinsic is observed after that
  intrinsic returns (`cooperative_pause_waits_for_a_blocking_intrinsic_to_return`).
  In-call interruption belongs to `U50-40`.
- Recoverable I/O errors must not drop the debug session unless disconnect is
  requested. Authentication here means the debug session owns the channel; it
  is not remote attach (`UMB-60`).

## `UMB-50B` — live terminal input and output

- Proven subset: stopped-state queued lines, EOF, cancel, and a cumulative
  session byte quota. Hosted debug `TextInput` never reads process stdin.
- Captured structured `output` events remain the program-output channel.
- Not in this subset: a second console/PTY (`live_terminal` stays false),
  `ReadKey`/TUI, or waiting inside an already blocked `ReadLn` (`U50-40`).

## `UMB-50C` — TUI and graph events

- Proven subset: debug hosted `KeyInput` never polls the process terminal.
  Queued TUI events and graph `On*` handlers run only as bytecode after
  continue. Stopped stack, variables, and evaluate do not dispatch them.
- JSONL `io.event` and DAP `fpas/event` are unsupported. There is no second
  editor event loop.
- In-call pause inside `Application.Run` or blocking `ReadEvent` remains
  `U50-40`.

## `UMB-50D` — pause inside blocking host calls

- Begins only after the transport and event-ownership contracts exist.
- Interruption must be cooperative. Killing an OS thread is forbidden.
- Current behavior (observe pause after the intrinsic returns) remains the
  documented bound until a proven in-call contract exists.

## Out of scope

- Attach/remote (`UMB-60`).
- Data breakpoints (`UMB-70`).
- Record/replay (`UMB-80`) and hot reload (`UMB-90`).
- Non-stop execution (rejected by `UMB-40D`).
