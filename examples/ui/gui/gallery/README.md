# FPas CUI GUI gallery

Build and run `GuiWidgetGallery.fpasproj` after the
[local SDK bootstrap](../../../../docs/build/development.md#build-an-fpas-application).
The [CUI build reference](../../../../src/Fpas.Ui.Gui.Cui/native/README.md#prerequisites)
lists the native prerequisites.

The gallery follows CUI's desktop composition with a full-height 240-point sidebar,
a centered content column, and a left drawer below 760 logical points. It shows
the accepted widget examples from the
[CUI inventory](../../../../docs/future/cui-gaps.md).

## Source structure

- `App/`: navigation, responsive window composition, root state and message routing.
- `Shared/`: common section composition and the owner of values used on several
  pages, including appearance, profile data and filter settings.
- `Pages/<Topic>/`: a page's state, messages, update and view. Stateful widget
  examples live in that page's `Widgets/` directory. Actions owns its background
  save commands. Overview is a static view with a supplied navigation message.
  Feedback is a static composition; its alerts retain their local state in CUI.

Namespaces follow these directories, for example
`GuiGallery.Pages.Actions.State` and `GuiGallery.Pages.Actions.Widgets.SplitButton`.
There is one MVU session. The root embeds child views with `Fpas.Ui.Composition.Map`
and translates their transitions with `Fpas.Ui.Mvu.Composition.MapTransition`.
Each feature receives a stable background-effect scope. Shared values have one
owner, so the profile dialog and Inputs, or Filters and Selection, stay in sync.

## Pages

- **Buttons** shows the variants, sizes, icons, and input states. Save changes
  shows the busy state, and Reset enables it again. The formatting group toggles
  three independent actions. The copy button copies the displayed command.
  Hold to delete demonstrates confirmation without deleting anything.
  Segmented control reproduces CUI's List/Board/Timeline icon choices and its
  small Day/Week/Month/Year selector, with independent model-owned selections.
  Dropdown menu clones CUI's Options and icon-only More actions examples,
  including section captions, separators, icons, shortcut hints and a danger
  action. Commands only update preview state; they do not modify a project.
- **Inputs** shows the original responsive Text fields card with Full name,
  Email, Password, Search projects and About. The email example uses a
  reserved example domain. Typing a short password displays the validation
  error. The multiline About field spans the card and wraps on narrow screens.
  The next card shows Topics, the six-box Verification code starting with
  418, and New password with its strength meter.
  Pickers adds Country, searchable Language, and Seats in their original
  grid positions, stacked at narrow widths.
- **Selection** shows native notification checkboxes, connectivity switches
  and the exclusive save-frequency radio group. The checkbox and switch
  cards share a row when both fit and stack at narrower widths. Radio cards
  offer three save destinations with Team server selected. Sliders show
  continuous Volume, Brightness in ten-point steps and a draggable Price range.
  The six category chips wrap on narrow screens. Design and Marketing start
  selected. The five-star rating starts at four stars and clears when the
  selected star is clicked again.
- **Navigation** shows underline tabs with model-selected content and pill
  tabs with icons. The breadcrumb shows the activated ancestor in a toast.
  The twelve-page pager keeps the current page, first page and
  last page visible on narrow screens. The stepper starts at Workspace and
  follows the Back and Next buttons. Finish shows an “All set” toast.
- **Overlays** shows the original Edit profile dialog with Name, Email and
  Public profile. The fields update shared preview values. Cancel closes the
  dialog and Save changes also shows a success toast. The same card opens the
  right-hand Notifications sheet. Escape, the close button and the scrim
  dismiss both overlays.
  The context-menu area opens CUI's Cut/Copy/Paste/Delete menu on right click.
  Its commands only update preview state. Arrow keys and Enter choose a command;
  Escape and outside clicks dismiss it.
  The Filters button opens a native popover with two checkboxes, a score slider,
  and Apply. The values are shared with the matching gallery controls.
  The Toasts section includes success, failure and update notifications and
  an Archive a message example with a timed Undo action.
  The Tooltips section reproduces the six CUI toolbar buttons, with formatting
  shortcuts shown as key hints on hover.
- **Lists** shows the original three-question native accordion.
- **Feedback** shows CUI's four original alerts, including dismissible storage
  warning, upload percentage, indeterminate indexing, Storage usage and three
  spinner sizes. Loading reproduces the native skeleton avatar, percentage text
  lines and body placeholder, with CUI-owned shimmer and reduced-motion behavior.
  Activity reproduces the three native deployment, review and
  failed-build entries. Goals shows the native 104-point Onboarding and Quota rings.
  Leaving the page releases native dismissal and interpolation state. Returning
  shows the warning again. The cards use the available width; the native
  two-column grid and Goals row justification remain open layout items in the
  CUI inventory.
- **Disclosure** shows clamped About text and Billing and Security expandable
  cards. The model owns header expansion;
  CUI owns the text link's local state. The original branch confirmation uses
  the danger action and Cancel button. All examples retain CUI rendering and
  interaction, including its [documented focus limitation](../../../../docs/future/cui-upstream-bugs.md#cui-006).
- **Actions** shows the primary Save and outline Export split buttons.
  Save reports its main or menu action beside the buttons.
  The Toggle group section shows independent formatting icons and exclusive
  text alignment, with the original Bold and Left defaults.
  Delete project demonstrates two-step confirmation and shows Project deleted
  beside the buttons after confirmation.
  Activate twice within three seconds. Escape or a click outside cancels.
  Save changes shows inline work feedback. It runs for 1.4 seconds, displays
  Saved for 1.6 seconds, and returns to idle.
  Reactions starts with four native icon pills and animates changes to their counts.
  The Theme section places the light/dark/System switcher beside the sun/moon
  toggle. The toggle proposes an explicit opposite light/dark
  choice, including when System was selected, and retains focus after changing.
  User menu opens the original account popup with Profile, Billing, Settings
  and Log out as preview commands.
- **Smart inputs** shows the original URL and weight fields with a prefix and
  suffix. Edits change only the value inside the field.
- The sidebar changes pages. At narrow widths, open it with the **Components**
  icon. Selecting a page dismisses the drawer.
  Theme segments use Tab, Left/Right, Enter, Space or the pointer; selection
  updates the model and its shared palette.

The GUI gallery contract runs this view with both native renderers and captures
the accepted pages and open overlays, including Lists plus light, dark and narrow
Disclosure and Feedback views. It drives the widgets with native pointer and keyboard input.
The same contract also drives the package-only consumer.
