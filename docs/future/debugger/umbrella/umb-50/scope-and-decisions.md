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
- `DebugSession::output()` retains captured stdout lines. `--output-limit`
  bounds the session. There is no live debuggee stdin channel distinct from
  protocol stdin.
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

## `UMB-50A` — protocol versus debuggee transport

- Begins only after `U50-00` records the current stdio sharing.
- A successful debuggee channel must not parse as JSONL or DAP. Protocol
  stdout must not contain raw program bytes.
- Lifecycle covers connect, disconnect, EOF, and recoverable I/O errors
  without dropping the debug session unless disconnect is requested.
- Authentication here means the debug session owns the channel; it is not
  remote attach (`UMB-60`).

## `UMB-50B` — live terminal input and output

- Begins only after `UMB-50A` can deliver bytes without protocol ambiguity.
- Input must be ordered, cancellable, and bounded. EOF is a first-class
  event. Process cleanup must not leak host handles.
- Existing captured `output` events remain the fallback until a live channel
  is proven.

## `UMB-50C` — TUI and graph events

- Event ownership must be deterministic while stopped and after resume.
- A stop freezes hosted event dispatch the same way it freezes bytecode:
  no hidden paint, key, or mouse handler runs during inspection.
- Reuse all-stop from `UMB-40A`; do not invent a second event loop.

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
