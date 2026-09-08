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
- Subscription replacement is cancel-and-join before start. Shutdown is cooperative and has no
  forced process-level escalation.

## Acceptance evidence

- The interactive host selects application messages alongside Console input and probes completed
  groups on its bounded 50 ms host timer.
- Alternating case priority bounds source preference under sustained ready traffic.
- The host cancels and joins subscriptions and one-shot work on replacement or shutdown.
- Headless regressions cover FIFO delivery, queue-full rejection, command execution, task failure,
  replacement cancellation, shutdown cancellation, and late-send rejection.
- The complete pre-existing TUI regression group remains the compatibility gate for keyboard,
  pointer, resize, focus, and rendering behavior.

No performance conclusion is attached to this slice; its acceptance gates are behavior and
lifecycle correctness.
