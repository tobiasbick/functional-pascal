# Dogfooding gaps

Intake list for gaps found while agents write their helper scripts in FPAS instead of Python,
JavaScript, or shell logic.
Compiler panics also follow the entry rule in
[compiler-panic-followups.md](compiler-panic-followups.md).

## Entry format

Add one entry per gap under [Open entries](#open-entries):

```markdown
### <Short title>

- **Date:** YYYY-MM-DD
- **Kind:** blocker (fell back to another language) or awkward (done in FPAS)
- **Area:** language, `Std.*` API, bug, diagnostics, or performance
- **Task:** what the helper script had to do
- **Missing or wrong:** what FPAS could not do or did badly
- **Minimal example:** the smallest FPAS source that shows the gap
- **Fallback:** the language or tool used instead, if any
```

Remove an entry once the gap is fixed and covered by tests and current documentation.

## Open entries

### Comparing contract execution requires separate source copies

- **Date:** 2026-10-24
- **Kind:** awkward (done in FPAS)
- **Area:** CLI, compiler contracts, performance
- **Task:** Measure the runtime cost of the seven application-level contract
  blocks in Into the Dungeon's client and server.
- **Missing or wrong:** Contracts run in every build mode. There is no CLI policy
  that omits both condition evaluation and checks from the complete program,
  including library artifacts. `fpas run --no-contracts` is not supported.
- **Minimal example:** A function with `requires Value >= 0;` and
  `ensures (Returned): Returned >= Value;` cannot currently be built or run with
  those checks disabled through a CLI option.
- **Fallback:** No other language. An FPAS helper copied sources and manifests
  into two isolated trees, removed only the game's contract blocks in one tree,
  and built both variants. Alternating timing runs and separate profiler runs
  compared the resulting images. Standard-library contracts remained enabled;
  this does not emulate a program-wide contract-disable option. Into the Dungeon
  records the [CLI policy proposal](https://github.com/tobiasbick/fpas-into-the-dungeon/blob/5cc871c9c7d9afe9f2c0f0b7bd7cc8a74db1295f/docs/fpas-wishes.md#completely-disable-contracts-through-the-cli)
  and the [measurement report with raw timing data](https://github.com/tobiasbick/fpas-into-the-dungeon/blob/5cc871c9c7d9afe9f2c0f0b7bd7cc8a74db1295f/docs/bench/contracts.md).

### Incremental process-pipe input for an FPAS DAP controller

- **Date:** 2026-10-24
- **Kind:** resolved by interactive process pipes
- **Area:** `Std.Proc`, `Std.Console`, streaming I/O
- **Task:** Write an FPAS controller that drives Into the Dungeon's real client
  through DAP terminal events and plays the seed-25 acceptance route.
- **Missing or wrong:** `Std.Proc` exposes blocking run/capture operations with
  fixed initial stdin, but no interactive child-process handles or pipe streams.
  Standard-pipe composition was tried with an FPAS sender and receiver connected
  by loopback TCP. `Console.ReadText()` is documented as line-buffered and also
  waits for a newline when stdin is redirected. DAP bodies are byte-counted JSON
  without a terminating newline. The receiver can therefore block on a complete
  body while the debugger waits for the next request. This is a missing streaming
  capability, not a claim that `ReadText()` violates its current contract.
- **Minimal example:** Pipe the following producer into the consumer with native
  process pipes. The producer writes `A` immediately, but the consumer receives
  its first character only after the delayed newline (measured about 1998 ms).

  ```pascal
  program PartialPipeSource;
  uses Std.Console as Console, Std.Time as Time;
  begin
    Console.WriteText('A');
    Time.Sleep(2000);
    Console.WriteLn('B');
  end.
  ```

  ```pascal
  program PartialPipeReader;
  uses Std.Console as Console, Std.Time as Time;
  begin
    const Started := Time.MonotonicMillis();
    const Character := Console.ReadText().Unwrap();
    Console.WriteLn('first=', Character, ', wait_ms=', Time.ElapsedMillis(Started));
  end.
  ```

- **Observed DAP failure:** The FPAS prototype handles initialization but stalls
  reading a complete 87-byte launch response. Its sender eventually times out
  waiting for that response; no gameplay input has been sent.
- **Goal:** Support incremental byte I/O on process pipes, with clear EOF,
  ownership, bounds, cancellation and process cleanup. A raw standard-stream API
  could also support composed processes; preserve the existing line-buffered
  console input semantics.
- **Resolution:** `Std.Proc.Start` now creates owned interactive processes.
  Nonblocking `ReadStdout`, `ReadStderr`, `WriteStdin`, `CloseStdin` and `TryWait`
  distinguish pending work, raw bytes and EOF. `Terminate`, `Close` and VM
  teardown own process-tree cleanup. Native integration tests exchange binary
  output without line terminators before child exit and verify cleanup after
  explicit termination, closure, normal VM completion and runtime failure.
  Console input retains its documented line-buffered behavior.

### Virtual terminal acquisition blocked behind DAP input

- **Date:** 2026-10-24
- **Kind:** resolved runtime bug
- **Area:** `Std.Console`, Windows, DAP
- **Task:** Play the seed-25 acceptance route through an FPAS DAP controller.
- **Missing or wrong:** `AcquireInteractiveTerminal` queried host stdin even for
  the debugger's queue-only input. On Windows, the query blocked behind the
  protocol reader's pending read. A terminal-using target did not reach its
  function breakpoint without further requests; a target stopped on entry did.
- **Minimal example:** Launch a program that acquires the terminal and calls a
  `Marker` procedure, set a function breakpoint on Marker, send
  `configurationDone`, and keep stdin open without sending another request.
  The native CLI regression test timed out before the fix.
- **Resolution:** Only OS-enabled key input may probe host stdin or enable raw
  mode. The real CLI transport regression now reaches the breakpoint without
  keepalive requests. Virtual terminal input remains owned by the debugger.

### Debugger console replaced explicit dimensions with host dimensions

- **Date:** 2026-10-24
- **Kind:** resolved runtime bug
- **Area:** virtual console, DAP resize, Windows terminal queries
- **Task:** Render and drive the real game client at an explicit DAP terminal size.
- **Missing or wrong:** Capturing debugger consoles followed host dimensions on
  ordinary text, cell and frame operations. This could overwrite a DAP resize
  and issued native terminal-size queries throughout rendering.
- **Minimal example:** Resize a capturing console to 60 by 22 while its host
  size is 90 by 31. ScreenWidth returned 90. The regression injects the host
  size through the console's existing test fixture.
- **Resolution:** Capturing debugger consoles use virtual default dimensions
  and retain explicit resize events. Native writer consoles still follow the
  host. The test also checks dimensions after text output and screen clearing.

### Sustained gameplay through DAP remains expensive

- **Date:** 2026-10-24
- **Kind:** awkward (done in FPAS)
- **Area:** debugger execution and performance investigation
- **Task:** Play the seed-25 settlement acceptance route through real terminal
  key events, without assigning game state or bypassing the client.
- **Observed:** Walking packets of 64 keys take minutes under DAP, even with a
  4-by-4 virtual terminal and a source breakpoint reached only by the final
  menu key. The debugger child consumes most of the CPU time; controller polling
  is not the dominant CPU consumer. This is an end-to-end observation, not an
  isolated debugger benchmark or an identified runtime cause.
- **Fallback:** No other language. The FPAS helper uses a small initial terminal,
  bounded key packets, confirmed positions and explicit save checkpoints. It
  resizes for rendered map, inventory and dialogue checks. Earlier conditional
  view breakpoints added inspection stops and were replaced with the menu source
  point, but execution remains slow.
- **Goal:** Make sustained continue practical for interactive FPAS applications.
  Measure debugger dispatch, scheduling, root publication and inspection costs
  before changing their behavior; provide profiling of the actual debug target
  rather than inferring its costs from the controller's `run --profile` report.
