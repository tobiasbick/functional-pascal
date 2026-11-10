# GUI host platform acceptance

The implemented host contract is in [Fpas.Ui.Gui](../../../src/Fpas.Ui.Gui/README.md).
The [counter](../../../examples/ui/gui/counter/README.md) and
[widget gallery](../../../examples/ui/gui/gallery/README.md) use the public MVU and
shared-widget application path.

Phase 6 still requires native Windows x64 evidence. A Linux/Xvfb run cannot
establish Windows event delivery or the OS modal resize-loop behavior.

- [ ] Build the solution and the counter/gallery with the Windows native toolchain.
- [ ] Run `GuiHostTests`, `CuiHostLayoutTests`, `CuiHostResourceTests` and the
  remaining binding tests under software and OpenGL rendering. Retain gallery
  screenshots in ignored `.temp-data` output.
- [ ] Verify native live-resize callbacks on Windows: cached-tree painting only,
  no Update/View/effect reentry, rooted callback lifetime, safe detachment and
  managed failures surfaced after returning from unmanaged code. Include
  resize while workers/cancellation callbacks are pending and owner teardown.
- [ ] Verify native close during asynchronous cleanup: continue handling redraws
  while both worker and cancellation gates are held, then release resources on
  the creating thread and reject late results.
- [ ] Resolve Windows findings before closing phase 6. Final packaged-host
  publication and OS-input end-to-end checks remain in
  [phase 8](verification-and-documentation.md).
