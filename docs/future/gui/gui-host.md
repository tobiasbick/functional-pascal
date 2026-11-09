# GUI MVU host

Depends on the [managed binding](managed-binding.md), [MVU core](shared-mvu.md)
and [widget contracts](shared-widgets.md). Implement `Fpas.Gui` as the FPAS
adapter from shared declarative views to CUI.

## Session and frame lifecycle

- [ ] Provide an application/session entry point accepting an initial model,
  update routine and typed view, matching the shared MVU contracts. Keep all
  transitions, view evaluation and CUI calls on the owning UI thread.
- [ ] Translate the current widget tree into CUI calls with stable scoped IDs.
  Reconcile focus, controlled values, selection and editor state after model
  changes, including controls that disappear or move between modal scopes.
- [ ] Queue native interactions as typed messages for processing after the frame.
  Rendering never directly mutates the application model or executes effects.
- [ ] Distinguish application view invalidation from CUI animation repaint.
  Animation redraws reuse the current view; they do not rerun update/effects.
  Reevaluate responsive view/layout when its actual dependencies change.
- [ ] Handle native live-resize callbacks without reentering Update or command
  execution. Such callbacks may redraw the current view; defer application
  messages until the owning loop can process them safely.
- [ ] Publish host snapshots only after a successful presentation. Expose
  semantic control information for tests without pretending the GUI is a cell grid.

## Input, wakeup and resources

- [ ] Implement the shared focus/modal/keyboard rules. Resolve CUI input into
  the canonical message contracts and make model state authoritative for text,
  checkboxes, list/table/tab selection and explicitly controlled scrolling.
- [ ] Own editor buffers and image/canvas resources by stable control identity.
  Release resources when controls disappear, assets change or the session closes.
- [ ] Wake an idle native loop when a background result or cancellation arrives.
  Use the binding's platform wake operation; do not mutate CUI from worker tasks
  or require continuous polling/rendering as the normal idle strategy.
- [ ] Handle close, rendering failure and exceptions with ordered MVU shutdown
  and UI-thread native cleanup. Keep exceptions out of unmanaged callback frames.
- [ ] Expose actual renderer selection for diagnostics/tests. Software fallback
  is usable; tests requesting OpenGL must identify it if fallback occurs.

## Acceptance

- [ ] Run a small public FPAS MVU GUI example with a counter, checkbox, UTF-8
  editing, asynchronous result, cancellation and a modal overlay.
- [ ] Verify that idle completion reaches Update, events arrive once, animations
  do not restart commands and late results after close have no effect.
- [ ] Cover focus restoration, editor/document replacement, resize, control
  removal and repeated open/close. Assert ownership of native resources.
- [ ] Exercise the host under software and OpenGL rendering on Linux, then on
  Windows. Use virtual displays for cloud integration checks and retain screenshots.

The low-level binding fixture remains a targeted integration test; user-facing
examples and documentation use this single MVU/declarative application path.
