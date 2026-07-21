# Std.Tui3 runtime, tasks, errors, and terminal cleanup

## UI task rule

The main FPAS task owns the Tui3 application host. `Update`, `View`, layout, paint, and
host operations run on the main task.

Calling them from a spawned task is a programming error. Tui3 v1 has no worker-side host operation;
the diagnostic says that worker-result delivery is unsupported until the planned data-only message
transport exists. It must not recommend a nonexistent closure callback escape hatch.

## Commands

Before every `Update`, the runtime initializes the mutable command output to `None`. After `Update`
returns the next model, the runtime handles that command before the next `View`. `Quit` stops the
loop without another paint or flush. This ordering is fixed for both headless and interactive runs.

```pascal
TuiCmd.None
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

## Headless hosts

`OpenForTest` creates a host with a private working surface and size and does not touch terminal
modes. Multiple headless hosts may exist in tests when they do not acquire the interactive terminal.
`SurfaceSnapshot` explicitly copies the last painted cells for assertions; routine layout and paint
do not pass that snapshot through the frame pipeline.
