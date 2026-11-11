# FPas CUI GUI gallery

Build and run `GuiWidgetGallery.fpasproj` after the
[local SDK bootstrap](../../../../docs/build/development.md#build-an-fpas-application).
The [CUI build reference](../../../../src/Fpas.Ui.Gui.Cui/native/README.md#prerequisites)
lists the native prerequisites.

The gallery follows CUI's desktop composition with a full-height 240-point sidebar,
a centered content column, and a left drawer below 760 logical points. It shows
the Overview, Buttons, Overlays, and Actions pages with the widgets checked in the
[CUI inventory](../../../../docs/future/cui-gaps.md).

- **Buttons** shows the variants, sizes, icons, and input states. Save changes
  shows the busy state, and Reset enables it again. The formatting group toggles
  three independent actions. The copy button copies the displayed command.
  Hold to delete demonstrates confirmation without deleting anything.
- **Overlays** opens the right-hand Notifications sheet. Escape, the close button,
  and clicking the backdrop dismiss it.
- **Actions** shows the primary Save and outline Export split buttons. Their
  main and menu actions update the inline result, as in the CUI gallery.
  Delete project demonstrates two-step confirmation with a harmless counter.
  Activate twice within three seconds; Escape or a click outside cancels.
  Save changes shows inline work feedback. Complete, Fail and Reset select its
  model-owned result; the status button itself accepts activation only while idle.
- The sidebar changes pages. At narrow widths, open it with the **Components**
  icon. Selecting a page dismisses the drawer.

The GUI gallery contract runs this view with both native renderers and captures
the Buttons page, the open Split button menu and successful inline work feedback.
It drives split, confirmation and status buttons with native keyboard input.
The same contract also drives the package-only consumer.
