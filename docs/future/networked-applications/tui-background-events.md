# Future: TUI Background Events

> Deferred. The current `Std.Tui` host remains terminal-input driven.

Interactive applications need to receive network, database, timer, and worker completions without
blocking terminal input or requiring application code to rebuild the TUI host. Updates and view
construction must still run serially on the main thread.

## Required interface

- Application-defined messages in addition to built-in keyboard, pointer, resize, focus, and
  control messages.
- A clonable, task-safe message sink that queues a bounded value and wakes the interactive host.
- Commands that start owned background work after `Update` returns rather than performing blocking
  work inside `Update`.
- Subscriptions for long-lived event sources with explicit cancellation during replacement or host
  shutdown.
- Headless injection of the same application-defined messages for deterministic tests.

## Ordering and ownership

- `Update`, command application, `View`, layout, and paint remain serialized on the main thread.
- Each source preserves FIFO ordering; ordering between independent sources must be documented.
- A full queue follows an explicit policy such as backpressure, coalescing for selected event kinds,
  or rejection. Silent loss is not permitted.
- Closing an application rejects new messages, cancels owned work, and wakes a blocked host.
- Background failures arrive as data that `Update` can handle rather than panicking an unrelated
  worker silently.

## Open design decision

The concrete type shape should be designed together with channels and cancellation. Candidate
interfaces include a generic application event wrapper or a separate application-owned inbox. The
choice must keep built-in routing convenient without forcing application payloads through the
closed `TuiMsg` variants.

## Acceptance requirements

- A background completion repaints an idle interactive application without terminal input.
- Terminal and background events remain responsive under sustained bounded traffic.
- Closing during blocked network, database, timer, and channel work cannot leak a task.
- Headless tests reproduce ordering, queue-full, cancellation, and late-delivery cases.
- Existing keyboard, pointer, resize, focus, and rendering behavior remains compatible unless a
  separate public-interface change is explicitly approved.
