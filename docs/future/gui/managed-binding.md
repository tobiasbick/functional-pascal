# Managed CUI binding

The initial [Fpas.Gui.Cui binding](../../../src/Fpas.Gui.Cui/README.md) provides
UI-thread-owned windows, scalar native option adapters, column layout, headings,
text, buttons, checkboxes and single-line editing. Its MSBuild target builds and
copies the shared native library and notices into referencing project outputs.

Linux tests cover explicit C interop, boolean/geometry values, UTF-8 editing
under compacting collections, Unicode/grapheme offset conversion, thread/frame
ownership, both renderers, resized capture and capture-failure recovery.
The library enables AOT compatibility analysis. Windows and the following work
remain open; full application/SDK publishing belongs to
[FPAS publishing](fpas-publishing.md).

## Remaining primitives and ownership

- [ ] Add the primitives required by the remaining shared widgets: rows, focus,
  list/table/tab selection, multiline editing, scrolling and overlays. Keep a
  single owned binding rather than exposing native option structs to FPAS.
- [ ] Connect the buffer's UTF-16/UTF-8 conversion to native editor caret and
  selection state, with controlled document replacement and multiline navigation.
  Preserve grapheme boundaries when native editing reports code-point positions.
- [ ] Add explicit ownership and UI-thread disposal for image/texture resources
  and other persistent allocations introduced by these controls. Review native
  string retention for each new widget instead of extending the current subset's
  frame-copy assumptions to the entire CUI API.
- [ ] Supply thread-safe wakeup for queued managed work. Post a platform event
  without mutating CUI state from a worker; cover wakeup racing with disposal.
- [ ] If native callbacks are introduced, root their state/delegates for their
  full lifetime, handle live-resize reentrancy and surface failures after leaving
  native frames. Managed exceptions must not cross the C ABI.

## Remaining acceptance

- [ ] Assert owned resource counts across repeated sessions, allocation failures
  and resource replacement. Disposal/state tests alone do not establish leak freedom.
- [ ] Verify idle background wakeup and shutdown races: completion after disposal
  must not touch a destroyed window or deliver a late application message.
- [ ] Verify the corresponding Windows calling convention, boolean/struct ABI,
  editing, lifetime and rendering cases on Windows.
- [ ] Exercise the finished binding in final NativeAOT consumers as part of the
  publishing work, including static linking and all newly added entry points.

The application-level API belongs to the [GUI host](gui-host.md); the low-level
binding and its fixtures do not define a second FPAS application model.
