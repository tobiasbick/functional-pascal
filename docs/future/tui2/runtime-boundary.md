# Std.Tui2 runtime, tasks, errors, and terminal cleanup

## UI task rule

The main FPAS task owns every live Tui2 application. All normal application, view, layout, action, canvas, and registry operations must run on the main task.

Calling them from a spawned task is a programming error with a diagnostic that instructs the caller to post work to the main task.

## Posting from workers

Implemented headless foundation: `TuiApplication.Post` stores parameterless callbacks in an
application-scoped FIFO queue. `Tick` and `RunIterations` drain it before desktop layout and after
`OnTick`. Posts added during a drain run in the following drain; closing the application discards
callbacks that have not started. A posted `Quit` skips the remaining iteration phases and closes at
the loop boundary.

The runtime later extends this to a generic typed main-task queue. Tui2 exposes it through this
conceptual API:

```pascal
function TuiApplication.Post(
  App: TuiApplication;
  Handler: procedure()
): boolean
```

Posts are FIFO. The closure is type-checked at the call site and type-erased in the VM queue. The
application event loop drains the queue before layout and after each bounded terminal wait. Worker
posts obey the transfer rules in [closures.md](../../pascal/language/functions/closures.md).

Posting is the only Tui2 operation permitted from a worker task. It returns `false` when application shutdown has begun; otherwise it enqueues the callback and returns `true`. Queued callbacks not started before shutdown are discarded. Posted handlers run on the main task and may use the normal Tui2 API.

This queue is implemented as a generic scheduler/runtime facility; it does not move widget behavior into Rust.

## Error categories

| Category | Contract |
| --- | --- |
| Expected user outcome | Return `Option` or `Result`, for example dialog cancellation or file selection failure. |
| Invalid caller input | Runtime diagnostic, for example negative dimensions or invalid indices. |
| Broken TUI invariant | Runtime diagnostic and application termination. |
| Terminal I/O failure | Runtime diagnostic preserving the operating-system context. |
| Callback panic | Stop dispatch and preserve the callback diagnostic as primary. |

Invalid handles, cross-application handles, wrong handle kinds, forbidden callback mutations, and non-main-task calls are programming errors rather than recoverable results.

## Terminal session ownership

Only one live interactive application may own the process terminal. Opening a second one fails.

Opening is transactional through `Std.Console.AcquireInteractiveTerminal`:

1. acquire terminal ownership;
2. enter raw mode when stdin is a TTY;
3. enter the alternate screen;
4. enable mouse, focus, and paste events;
5. hide the cursor;
6. create the desktop and run `OnStart`.

If any step fails, completed steps are rolled back in reverse order. Without a terminal writer the
call records ownership only, so headless and CI hosts can still open an interactive application
handle without changing modes.

Orderly close runs `OnStop`, then `ReleaseInteractiveTerminal` restores cursor, paste, focus, mouse,
alternate screen, and raw mode in reverse acquisition order. Close is idempotent.

## Runtime safety net

The console host tracks every terminal mode the interactive session enabled. Console Drop restores
owned screen modes even when FPAS code panics, a callback fails, or application close is skipped.
Raw mode remains restored by `KeyInput` Drop when it was enabled.

Safety cleanup does not invoke user callbacks. Cleanup failures are secondary diagnostics and never
replace the original runtime failure.

This host cleanup is resource safety, not a TUI implementation: event routing, layout, controls,
actions, and painting remain FPAS source.
