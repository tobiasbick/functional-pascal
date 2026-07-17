# Std.Tui2 runtime, tasks, errors, and terminal cleanup

## UI task rule

The main FPAS task owns every live Tui2 application. All normal application, view, layout, action, canvas, and registry operations must run on the main task.

Calling them from a spawned task is a programming error with a diagnostic that instructs the caller to post work to the main task.

## Posting from workers

The runtime adds a generic typed main-task queue. Tui2 exposes it through this conceptual API:

```pascal
function TuiApplication.Post(
  App: TuiApplication;
  Handler: procedure()
): boolean
```

Posts are FIFO. The closure is type-checked at the call site and type-erased in the VM queue. The
application event loop drains the queue before layout and after each bounded terminal wait. Worker
posts obey the transfer rules in [capturing-closures.md](../capturing-closures.md).

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

Opening is transactional:

1. acquire terminal ownership;
2. enter raw mode;
3. enter the alternate screen;
4. enable mouse, focus, and paste events;
5. hide the cursor;
6. create the desktop and run `OnStart`.

If any step fails, completed steps are rolled back in reverse order.

Orderly close runs `OnStop`, then restores cursor, paste, focus, mouse, alternate screen, and raw mode in reverse acquisition order. Close is idempotent.

## Runtime safety net

The console host tracks every terminal mode it enabled. VM teardown restores all tracked modes even when FPAS code panics, a callback fails, or application close is skipped.

Safety cleanup does not invoke user callbacks. Cleanup failures are secondary diagnostics and never replace the original runtime failure.

This host cleanup is resource safety, not a TUI implementation: event routing, layout, controls, actions, and painting remain FPAS source.
