# GUI widget gallery

The FPAS application uses `Fpas.Ui.Gui.Application` with the shared MVU and widget
contracts. Build or run `GuiWidgetGallery.fpasproj` with `dotnet build` or
`dotnet run --project`. Native source prerequisites are listed in the
[CUI build reference](../../../../src/Fpas.Ui.Gui.Cui/native/README.md#prerequisites).

Choose a page with the pointer or Tab and arrow keys. Controls demonstrate
button variants, disabled states, checkboxes and progress. Editors support
Unicode documents and selection. Collections show actions, keyed selection,
tables, tabs and an empty list. Layout shows fixed/fill sizing, borders, semantic
roles and controlled scrolling. Graphics uses owned RGBA pixels and vector
operations. Effects demonstrates asynchronous completion, cancellation and a
modal; the increment button remains usable while work runs. Escape dismisses
the modal and restores focus. Switch theme and resize without resetting state.

`GalleryState.fpas` owns transitions and effects, `GalleryView.fpas` navigation
and responsive composition, and `Previews.fpas` the widget inventory. Control IDs
are stable; collection keys belong to their containing ID.

Managed source publication uses the normal [FPAS publish modes](../../../../docs/build/publishing.md).
For application publication, reference the Fpas.Ui.Gui package from the matching
distribution feed. Its transitive CUI package supplies prebuilt shared and static
libraries. NativeAOT links CUI into the executable; managed single-file delivery
extracts the bundled native library. GuiPublishingTests exercises the complete
gallery in all five modes with both renderers, controlled values, theme,
selection, images, effects and repeated cleanup.
A headless Linux run needs a virtual display such as Xvfb.
