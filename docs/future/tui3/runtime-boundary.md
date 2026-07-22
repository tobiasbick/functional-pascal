# Std.Tui3 runtime, tasks, errors, and terminal cleanup

## UI task rule

The main FPAS task owns the Tui3 application host. `Update`, `View`, layout, paint, and
host operations run on the main task.

Calling them from a spawned task is a programming error. Tui3 v1 has no worker-side host operation;
the diagnostic says that worker-result delivery is unsupported until the planned data-only message
transport exists. It must not recommend a nonexistent closure callback escape hatch.

## Commands

Before every `Update`, the runtime resets its `TuiCmdOutput` capability to `NoCommand`. `Update`
sets the capability, and the runtime reads it after receiving the next model but before the next
`View`. `Quit` stops the loop without another paint or flush. This ordering is fixed for headless
and interactive runs.

```pascal
TuiCmd.NoCommand
TuiCmd.Quit
```

## Deferred effects and worker results

Arbitrary procedure-valued `Post`, `Batch`, timers, file dialogs, subscriptions, and asynchronous
worker results are not part of v1. A later design must answer all of these before implementation:

- the command and result remain data rather than arbitrary main-task code;
- the result becomes an application-consumable message without a generic `TuiElement` type;
- shutdown, FIFO ordering, and rejected posts have deterministic contracts;
- no path mutates the model, element tree, or working surface outside `Update` and paint.

Until that design passes its own FPAS spike, applications must not represent unsupported effects by
mutating private TUI globals or capturing model state in callbacks.

## Error categories

| Category | Contract |
| --- | --- |
| Expected user outcome | Represented in `Model` (for example cancelled confirm). |
| Illegal caller input | Runtime diagnostic (negative sizes, invalid control/action id). |
| Broken TUI invariant | Runtime diagnostic and application termination. |
| Terminal I/O failure | Runtime diagnostic with operating-system context. |
| Panic inside `Update` / `View` / paint | Stop the loop; preserve the diagnostic as primary. |

## Terminal session ownership

Only one live interactive application may own the process terminal. Opening a second one
fails.

Opening is transactional through `Std.Console.AcquireInteractiveTerminal`:

1. acquire terminal ownership;
2. enter raw mode when stdin is a TTY;
3. enter the alternate screen;
4. enable mouse, focus, and paste events as needed;
5. hide the cursor;
6. enter the run loop.

If any step fails, completed steps roll back in reverse order. Closing or process teardown
restores terminal modes. The VM keeps a safety net for abrupt termination.

### Phase 5 boundary audit

Gate 5.A is complete. Tui3 calls only these existing `Std.Console` operations:

- `AcquireInteractiveTerminal` / `ReleaseInteractiveTerminal` for exclusive ownership and
  reverse-order mode restoration;
- `ScreenWidth` / `ScreenHeight` to create the first working surface;
- `ReadEventTimeout` for bounded input waits;
- `BeginFrame`, `WriteCells`, and `Present` to flush one painted surface.

`Runtime/TerminalSession.fpas` owns the Tui3-side acquire/release pairing;
`Runtime/ConsoleEvents.fpas` maps supported Console events to host input; and
`Runtime/TerminalRenderer.fpas` resolves palette colors and emits Console cells. The Console
cell API has glyph and foreground/background colors but no text-attribute fields, so Bold, Dim,
Underline, and Inverse do not alter terminal output in this phase. No compiler, VM, or
`Std.Tui` bridge change is required.

## Headless hosts

`OpenForTest` creates a host with a private working surface and size and does not touch terminal
modes. Multiple headless hosts may exist in tests when they do not acquire the interactive terminal.
`SurfaceSnapshot` explicitly copies the last painted cells for assertions; routine layout and paint
do not pass that snapshot through the frame pipeline.
