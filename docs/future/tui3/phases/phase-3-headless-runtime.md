# Phase 3 — Headless MVU hardening

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).
The core host, queue, focus, Button/Input messages, commands, and confirm-dialog slice are already
complete. Do not recreate them.

## Task 3.1 — Add resize and pointer host input

**Status:** ready.

**Files:** add a focused pointer value file under `lib/Std/Tui3/Runtime/`; modify
`Runtime/Input.fpas`, `Runtime/Msg.fpas`, `Runtime/Application.fpas`, and `lib/Std/Tui3.fpas`; add
`tests/stdlib/tui3/resize_message_test.fpas` and
`tests/stdlib/tui3/pointer_fallback_test.fpas`.

**Contract:** add the `Pointer` and `Resize` messages already listed in
[api-surface.md](../api-surface.md). Test injection uses normalized zero-based application
coordinates. Resize changes host size before its message reaches `Update`. Unhandled pointer input
remains a raw `TuiMsg.Pointer`.

**Done:** FIFO ordering with injected messages/ticks is covered; headless injection never touches
terminal state.

## Task 3.2 — Route pointer input through arranged geometry

**Status:** blocked by Task 3.1.

**Files:** split `Runtime/Routing.fpas` into `Runtime/Routing/Focus.fpas`, `Key.fpas`, `Pointer.fpas`,
and a small module facade before adding pointer logic; add
`tests/stdlib/tui3/pointer_routing_test.fpas` and
`tests/stdlib/tui3/modal_routing_test.fpas`.

**Contract:** use the exact ordering in [event-loop.md](../event-loop.md): foremost modal subtree,
topmost half-open hit, focus change before action/value message, raw fallback. Pointer routing uses
the arranged frame and never derives geometry independently.

**Done:** overlapping clips, borders, modal exclusion, focus-before-action, and fallback are
covered. Keyboard tests remain unchanged after the split.

## Task 3.3 — Complete runtime failure and ordering canaries

**Status:** blocked by Task 3.2.

**Files:** modify only the owning runtime modules; add
`tests/stdlib/tui3/quit_order_test.fpas`, `message_fifo_test.fpas`, and targeted Rust tests only if
the diagnostic originates in the compiler or VM.

**Contract:** enforce [event-loop.md](../event-loop.md) and
[runtime-boundary.md](../runtime-boundary.md): pending messages precede new input, one message
consumes one iteration, `Cmd` resets before every update, Quit prevents another View/paint, and
panic keeps the primary diagnostic.

**Done:** all ordering canaries pass for both direct injection and routed input. No `Post`, `Batch`,
closure callback, or asynchronous-result claim is added.

## Phase checkpoint

The existing confirm-dialog tests plus Tasks 3.1–3.3 cover the complete headless runtime contract.
