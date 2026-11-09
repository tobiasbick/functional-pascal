# Shared MVU core

Extract `Fpas.Mvu` from the implemented [TUI host](../../../src/Fpas.Tui/README.md).
Use an FPAS library and ordinary CLR types so FPAS, C# and F# consumers can use
the same runtime contracts. It must not depend on widget, terminal or CUI types.

## Extraction

- [ ] Move typed model/message/update transitions and command/effect ownership
  into the shared core. Retain one implementation rather than copying the TUI
  session into a GUI session.
- [ ] Preserve bounded FIFO delivery, observable full/closed outcomes, one
  application message per transition, effect replacement by ID, cancellation,
  failure routing and rejection of stale results already queued by old effects.
- [ ] Keep the initial model and update routine independent of view construction.
  Transitions replace the model and schedule commands once; presentation and
  animation redraws must not rerun updates or restart effects.
- [ ] Separate host commands such as focus and close from backend-neutral
  effect execution. Hosts resolve control IDs and own their input/view/presenter
  state; the core never imports terminal keys, cell sizes or native handles.
- [ ] Define scheduling/wakeup and lifecycle hooks that both hosts can implement.
  Background work posts messages; model transitions run on the owning host thread.
- [ ] Define close/failure ordering: stop new work, cancel owned effects, reject
  late posts, join cooperative work before releasing resources it can use, and
  make cleanup observable. Cancellation does not force uncooperative code to stop.

## Host adaptation

- [ ] Adapt the TUI session to the shared core while retaining its headless
  driver, input routing, layout/render caches and successful-presentation boundary.
- [ ] Preserve the TUI rule that a failed presentation retains the last complete
  published frame; do not turn shared model commits into premature frame publication.
- [ ] Provide a deterministic core driver for transitions and effect tests that
  needs neither a terminal nor a window system.

## Acceptance

- [ ] Cover message ordering/backpressure, effect replacement, cancellation,
  failures, stale queued results and shutdown with deterministic tests.
- [ ] Confirm one update/effect execution per accepted message. Repaint, layout
  and animation do not replay messages; explicitly generated messages such as
  a canvas resize still run Update under the same delivery rules.
- [ ] Run the affected TUI session/effect/headless contracts and CLR consumer tests.
- [ ] Demonstrate the same model/update/effects under a headless driver and the
  adapted TUI. Reuse them under the GUI when its [host](gui-host.md) is available.

This library extracts the existing MVU command system. It does not depend on the
separate [language effect-system proposal](../effect-system-gap.md).
