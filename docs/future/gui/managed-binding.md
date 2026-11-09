# Managed CUI binding

The initial [Fpas.Gui.Cui binding](../../../src/Fpas.Gui.Cui/README.md) provides
UI-thread-owned windows, scalar native option adapters, row and column layout,
programmatic focus and single-line editing. It supports headings, text, buttons,
checkboxes and controlled selection in lists, tables and tabs.
Its MSBuild target builds and copies the shared native library and notices into
referencing project outputs.

The [binding reference](../../../src/Fpas.Gui.Cui/README.md#source-build-and-verification)
covers test scope and platform verification. Application and SDK publishing
belong to [FPAS publishing](fpas-publishing.md).

## Remaining primitives and ownership

- [ ] Add multiline editing, scrolling and overlays
  to the owned binding. Keep native option structs behind the binding.
- [ ] Connect the buffer's UTF-16/UTF-8 conversion to native editor caret and
  selection state, with controlled document replacement and multiline navigation.
  Preserve grapheme boundaries when native editing reports code-point positions.
- [ ] Add explicit ownership and UI-thread disposal for image/texture resources
  and other persistent allocations introduced by these controls. Review native
  string retention for each new widget instead of extending the current subset's
  frame-copy assumptions to the entire CUI API.
- [ ] If native callbacks are introduced, root their state/delegates for their
  full lifetime, handle live-resize reentrancy and surface failures after leaving
  native frames. Managed exceptions must not cross the C ABI.

## Remaining acceptance

- [ ] Assert owned resource counts across repeated sessions, allocation failures
  and resource replacement. Disposal/state tests alone do not establish leak freedom.
- [ ] Exercise the finished binding in final NativeAOT consumers as part of the
  publishing work, including static linking and all newly added entry points.

The application-level API belongs to the [GUI host](gui-host.md); the low-level
binding and its fixtures do not define a second FPAS application model.
