# TUI Background Events

> Implemented. The current contract is documented in
> [`Std.Tui` application host](../../pascal/std/tui/application.md).

Interactive applications can receive network, database, timer, and worker completions without
blocking terminal input or rebuilding the TUI host. Updates and view construction remain serialized
on the host task.

## Implemented interface

- `RunWithBackground` and `RunWithBackgroundAndPalette` receive application-defined messages from a
  bounded caller-created `channel of TMessage` and wake without terminal input.
- `TuiMsg.Started` lets the first serialized update queue work after the initial frame.
- `StartBackground`, `ReplaceSubscription`, and `CancelSubscription` transfer task ownership to the
  host after an update returns.
- `RunBackgroundIterations`, `InjectBackgroundForTest`, and `CloseWithBackground` expose the same
  bounded delivery and lifecycle rules to deterministic headless tests.
- `BackgroundFailed` forwards task-group failures as data while another update is possible.

## Ordering and ownership

- Framework update, application-message update, command application, `View`, layout, and paint
  remain serialized on the host task.
- Each channel preserves FIFO order. Routed framework messages drain first; the interactive host
  alternates Console/application case priority when both sources stay ready.
- `SendWithCancellation` supplies backpressure. `TrySend` and headless injection return `Ok(false)`
  when full. Neither path silently drops a value.
- Closing cancels and joins owned work before closing the application inbox, so blocked sends wake
  and late sends fail.
- Each operation owns one task group and is reaped after completion, avoiding retained completed
  children. Applications are limited to 256 simultaneous operations.

## Settled design decisions

- Application payloads stay in a separate generic channel rather than extending the closed
  framework message with application-specific variants.
- Framework lifecycle and failures use `Started` and `BackgroundFailed` variants on `TuiMsg`.
- Work uses the fixed `function(Token): result of boolean, string` shape. Its success value is
  ignored; errors are normalized through task-group failure records.
- Subscription replacement requests cancellation without blocking the host. The host joins completed
  sources and starts only the latest pending replacement, preserving stop-before-start ordering.
  Shutdown discards replacements, joins existing work cooperatively, and has no forced escalation.

## Acceptance evidence

- The canonical Mandelbrot TUI uses a bounded typed frame inbox and one replaceable render subscription.
  One task per row uses the VM worker pool; each publication contains a complete image.
  Cancellation checks use iteration chunks, and the serialized application update ignores stale images.
  Navigation retains the previous image until its replacement is ready when dimensions are unchanged.
- The interactive host selects application messages alongside Console input and probes completed
  groups on its bounded 50 ms host timer.
- Alternating case priority bounds source preference under sustained ready traffic.
- The host requests subscription cancellation without joining on the input/paint path, then reaps
  completed groups. Shutdown joins subscriptions and one-shot work without admitting replacements.
- Headless regressions cover FIFO delivery, queue-full rejection, command execution, task failure,
  replacement cancellation, shutdown cancellation, and late-send rejection. Additional regressions
  cover three updates and paint while an old worker is gated, latest-request coalescing, cancellation
  of pending work, and failure delivery from a stopping source.
- The complete pre-existing TUI regression group remains the compatibility gate for keyboard,
  pointer, resize, focus, and rendering behavior.

## Responsiveness correction

The first Mandelbrot migration limited rendering to four tasks and repainted after every row.
It also exposed synchronous subscription joining on the UI path. These issues are corrected by
VM-pool row tasks, atomic image updates, and deferred subscription reaping.
The root `Select` wait also leaves queued computations to pool workers, so a slow row cannot
take over the waiting input task. A VM regression verifies that queued work stays queued.
The Console reader yields after every poll. At sixteen-pixel checkpoints, Mandelbrot tasks
yield after at least eight milliseconds of work.
An interactive-host regression waits for a background message and exits without terminal input.
That regression and both Mandelbrot regressions also pass with only one CPU available to the process.
The complete FPAS suite passes with 425 passed, one intentionally skipped, and no failures.
The repeatable `mandelbrot` benchmark group covers result delivery with and without headless
grid painting; measurements and their limits are recorded in [benchmark history](../../bench/history.md).

No public CPU-count or RAM API was added. The existing pool already derives its size from
available parallelism, leaving one logical processor for the root where possible and retaining
at least one worker. Broader [hardware information](../hardware-information.md) is recorded only
as an idea, not an implementation plan.
