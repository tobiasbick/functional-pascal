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
  Segmented control reproduces CUI's List/Board/Timeline icon choices and its
  small Day/Week/Month/Year selector, with independent model-owned selections.
  Dropdown menu clones CUI's Options and icon-only More actions examples,
  including section captions, separators, icons, shortcut hints and a danger
  action. Commands only update preview state; they do not modify a project.
- **Overlays** opens the right-hand Notifications sheet. Escape, the close button,
  and clicking the backdrop dismiss it.
  The context-menu area opens CUI's Cut/Copy/Paste/Delete menu on right click.
  Its commands only update preview state. Arrow keys and Enter choose a command;
  Escape and outside clicks dismiss it.
- **Actions** shows the primary Save and outline Export split buttons. Their
  main and menu actions update the inline result, as in the CUI gallery.
  Delete project demonstrates two-step confirmation with a harmless counter.
  Activate twice within three seconds; Escape or a click outside cancels.
  Save changes shows inline work feedback. Complete, Fail and Reset select its
  model-owned result; the status button itself accepts activation only while idle.
  The Theme section uses the same light/dark/System switcher as the sidebar.
  Its source button explicitly simulates an application appearance-provider change.
  The adjacent sun/moon Theme toggle proposes an explicit opposite light/dark
  choice, including when System was selected, and retains focus after changing.
- The sidebar changes pages. At narrow widths, open it with the **Components**
  icon. Selecting a page dismisses the drawer.
  Theme segments use Tab, Left/Right, Enter, Space or the pointer; selection
  updates the model and its shared palette.

The GUI gallery contract runs this view with both native renderers and captures
the Buttons page, the open Split button menu, inline work feedback and the
appearance controls. It drives split, confirmation, status and theme actions
with native keyboard input.
The same contract also drives the package-only consumer.
