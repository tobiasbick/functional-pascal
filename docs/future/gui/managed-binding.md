# Managed CUI binding

Depends on [native integration](native-integration.md). Implement
`Fpas.Gui.Cui` as a C# library compatible with .NET 10 and NativeAOT. It owns
native lifetime and marshalling; application model/update logic remains outside it.

## Interop contract

- [ ] Use source-generated `LibraryImport` with explicit C calling conventions
  for every entry point. Represent C booleans with the correct one-byte ABI;
  verify struct layout for any directly imported value types on both targets.
- [ ] Import ordinary CUI functions directly. Use the small C bridge for macros
  and complex option setup instead of duplicating native structs throughout C#.
- [ ] Keep pointers and native allocation details out of FPAS-facing types.
  Expose ordinary CLR values, owned handles and bounded buffers.
- [ ] Implement the primitives needed by the planned shared widgets, including
  layout, controls, focus, selection, editing, scrolling and overlays. Scope the
  first integration fixture to a window, button, checkbox and text field.

## Lifetime, threading and text

- [ ] Own windows, contexts, textures and native buffers explicitly. Dispose
  graphics resources on their UI thread, including on failure; never call native
  graphics teardown from an arbitrary managed finalizer thread.
- [ ] Keep UTF-8 editing buffers stable for the required native lifetime.
  Validate sizes/terminators and distinguish borrowed, frame-local and persistent
  memory. Review string retention for each bound widget rather than assuming
  every CUI call copies temporary marshalling buffers.
- [ ] Keep application-visible strings and document positions in CLR form.
  Provide explicit conversion between UTF-8 byte positions and UTF-16 offsets,
  preserving valid scalar/grapheme boundaries and editor selection semantics.
- [ ] Keep all CUI mutation on the UI thread. If native callbacks are introduced,
  root their state/delegates for their full lifetime and return exceptions to the
  managed host after leaving native frames; exceptions must not cross the C ABI.
- [ ] Supply a thread-safe wake operation for queued managed work. CUI's app
  layer can sleep awaiting native input; waking it must post a platform event
  without calling ordinary CUI state APIs from a worker thread.

## Acceptance

- [ ] Exercise managed-to-native calls and returned changes for a button,
  checkbox and UTF-8 editor, including compacting managed collections between frames.
- [ ] Test empty/non-ASCII text, buffer limits, position conversion and repeated
  create/dispose cycles. Assert that owned handles/buffers are released on failure.
- [ ] Verify idle wakeup and shutdown races: completion after disposal must not
  access a destroyed native window or deliver a late application message.
- [ ] Build with AOT compatibility analysis and no reflection/code-generation
  workaround for the interop layer. Verify Linux first, then Windows.

The application-level GUI API belongs to the [GUI host](gui-host.md), not to a
second FPAS rendering loop built around this low-level binding.
