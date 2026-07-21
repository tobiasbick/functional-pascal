# Std.Tui3 runtime, tasks, errors, and terminal cleanup

## UI task rule

The main FPAS task owns the Tui3 application host. `Update`, `View`, layout, paint, and
host operations run on the main task.

Calling them from a spawned task is a programming error with a diagnostic that tells the
caller to request work through `TuiCmd.Post` or a later typed main-task queue.

## Commands and posting

`Update` returns `TuiCmd`. The runtime executes commands after `Update` and before the next
`View` (or at defined drain points around paint — exact ordering is fixed during
implementation and must stay deterministic for tests).

```pascal
TuiCmd.None
TuiCmd.Quit
TuiCmd.Batch(Commands)
TuiCmd.Post(Handler: procedure())
```

`Post` enqueues a parameterless main-task callback. FIFO. Closures obey
[capture transfer rules](../../pascal/language/functions/closures.md). Posting is the only
Tui3 host operation permitted from a worker. It returns `false` once shutdown began.
Callbacks not started before shutdown are discarded.

Posted callbacks may inject messages or request quit only through the host API — they must
not paint directly.

This queue is a scheduler facility. It does not move widget behavior into Rust.

## Error categories

| Category | Contract |
| --- | --- |
| Expected user outcome | Represented in `Model` (for example cancelled confirm). |
| Illegal caller input | Runtime diagnostic (negative sizes, invalid action id). |
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

`OpenForTest` creates a host with a surface and size and does not touch terminal modes.
Multiple headless hosts may exist in tests when they do not acquire the interactive
terminal.
