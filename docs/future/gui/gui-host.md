# GUI MVU host

Depends on the [managed binding](../../../src/Fpas.Gui.Cui/README.md), [MVU core](../../../src/Fpas.Mvu/README.md)
and [widget contracts](../../../src/Fpas.Ui/README.md). Implement `Fpas.Gui` as the FPAS
adapter from shared declarative views to CUI.

## Session and frame lifecycle

- [ ] Provide an application/session entry point accepting an initial model,
  update routine and typed view, matching the shared MVU contracts. Keep all
  transitions, view evaluation and CUI calls on the owning UI thread. Keep
  `Update` and `View` short. Schedule I/O and long computations as effects.
- [ ] Translate the current widget tree into CUI calls with stable scoped IDs.
  Reconcile focus, controlled values, selection and editor state after model
  changes, including controls that disappear or move between modal scopes.
- [ ] Run `Fpas.Ui.Validation.Prepare` with the graphical profile before issuing
  widget calls. Check any CUI-specific option limits against the resolved tree
  before drawing. Use the resolved cache for rendering and retain it after
  successful presentation.
- [ ] Queue native interactions as typed messages for processing after the frame.
  Rendering never directly mutates the application model or executes effects.
- [ ] Distinguish application view invalidation from CUI animation repaint.
  Animation redraws reuse the current view; they do not rerun update/effects.
  Reevaluate responsive view/layout when its actual dependencies change.
- [ ] Handle native live-resize callbacks without reentering Update or command
  execution. Such callbacks may redraw the current view; defer application
  messages until the owning loop can process them safely. Root callback state
  and delegates until detachment. Surface managed failures after returning
  from the native callback.
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
- [ ] Handle close, rendering failure and exceptions with the
  [MVU shutdown protocol](../../../src/Fpas.Mvu/README.md#cancellation-and-shutdown). Keep the
  native event loop responsive while cancellation and worker cleanup finish.
  Wake it on shutdown completion and release CUI resources on the owner thread.
  Keep exceptions out of unmanaged callback frames.
- [ ] Expose actual renderer selection for diagnostics/tests. Software fallback
  is usable; tests requesting OpenGL must identify it if fallback occurs.

## Widget gallery example

- [ ] Add `examples/gui/gallery/` with its own `GuiWidgetGallery.fpasproj`,
  following the [TUI gallery](../../../examples/tui/gallery/README.md). Write
  the application in FPAS using the canonical MVU host and widget contracts,
  with focused model/update, view and preview units.
- [ ] Provide widget navigation and interactive previews for every implemented
  GUI widget and supported variant, including disabled, empty and selected
  states. Keep the gallery aligned when adding or changing widget support.
- [ ] Cover text editing, lists/tables/tabs, progress, scrolling, overlays and
  layout, plus available GUI-only image/canvas extensions. Show current values
  and demonstrate asynchronous completion and cancellation.
- [ ] Support keyboard and pointer navigation, responsive window resizing and
  switching supported themes. Preserve page selection and preview model state
  across resize/theme changes. Reuse portable TUI gallery state/transitions
  where useful; graphical presentation follows the GUI's own layout units.
- [ ] Document how to build, run, navigate and publish the gallery in its README.
  Provide stable control IDs and deterministic scenarios for GUI-host tests.

## Acceptance

- [ ] Run a small public FPAS MVU GUI example with a counter, checkbox, UTF-8
  editing, asynchronous result, cancellation and a modal overlay.
- [ ] Verify that idle completion reaches Update, events arrive once, animations
  do not restart commands and late results after close have no effect.
- [ ] In a real CUI window, hold a background effect and then its cancellation
  callback at test gates. Click a second button and verify its state is painted
  before releasing each gate. During normal close with cleanup still pending,
  verify that the window handles a redraw before final teardown.
- [ ] Cover focus restoration, editor/document replacement, resize, control
  removal and repeated open/close. Assert ownership of native resources.
- [ ] Verify GUI extension rendering, resource ownership and diagnostics for
  unsupported host options at the adapter boundary.
- [ ] Exercise the host under software and OpenGL rendering on Linux, then on
  Windows. Use virtual displays for cloud integration checks and retain screenshots.
- [ ] Build and run the FPAS gallery, verify navigation and representative edits
  on its pages, and ensure previews cover the implemented GUI widget inventory.

The low-level binding fixture remains a targeted integration test; user-facing
examples and documentation use this single MVU/declarative application path.
