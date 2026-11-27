# Actions

Buttons, command menus, and controlled selections from the FPAS gallery's
Buttons and Actions pages.

| Widget | When to use it | Hosts |
| --- | --- | --- |
| [Button](button.md) | Run a single action such as saving a document or opening a view. | GUI and TUI |
| [Icon button](icon-button.md) | Use a glyph for a compact, familiar action. Give it a tooltip that names the action. | GUI |
| [Button group](button-group.md) | Join related actions in a single horizontal frame with dividers. | GUI |
| [Copy button](copy-button.md) | Send literal text to the clipboard and show brief feedback next to the copied value. | GUI |
| [Hold to confirm](hold-button.md) | Require a deliberate hold before a destructive or hard-to-undo action. | GUI |
| [Segmented control](segmented-control.md) | Choose one view or filter from a compact row of alternatives. | GUI |
| [Dropdown menu](dropdown-menu.md) | Open a list of commands from a labeled or icon-only trigger. | GUI |
| [Split button](split-button.md) | Put the primary action and related commands on the same control. | GUI |
| [Toggle group](toggle-group.md) | Group independent flags such as text formatting or exclusive choices such as alignment. | GUI |
| [Confirm button](confirm-button.md) | Ask for a second activation before carrying out a destructive action. | GUI |
| [Status button](status-button.md) | Show the progress and outcome of work in the button that started it. | GUI |
| [Reactions](reactions.md) | Show counts and the current user's reaction alongside a message or activity. | GUI |
| [Theme switcher](theme-switcher.md) | Select Light, Dark or an application-provided System appearance. | GUI |
| [Theme toggle](theme-toggle.md) | Switch between two concrete appearances with a single button. | GUI |
| [User menu](user-menu.md) | Put account commands behind a trigger that shows the user name and initials. | GUI |
| [Swipe actions](swipe-actions.md) | Reveal actions next to a list row without taking up space in its resting layout. | GUI |
| [Speed dial](speed-dial.md) | Group closely related commands behind one floating action button. | GUI |

Use the same stable IDs across views. Widgets propose typed messages when
the user acts. Store accepted changes in the model and apply them in the
next view. In menu and button options, shortcut captions display a hint.

## Buttons gallery page

The six sections use native cards. Button rows wrap at the width available
inside each card. The formatting group demonstrates joined actions. The copy
button copies the command, and a completed hold shows a danger toast. The two
segmented controls keep independent selections. Delete in the Options menu
opens the shared [project confirmation dialog](../overlays/dialog.md).

- [CUI Buttons comparison](../../../future/cui-compare/README.md#buttons-evidence)

## Actions gallery page

The Split button, Toggle group and Morphing buttons sections use wrapping
rows with a gap of 12 logical units. Save displays its most recent action.
Delete project displays a result after confirmation. The status button runs
through Idle, Busy and Success before returning to Idle.

The appearance controls share a wrapping row with a gap of 16 logical units.
The account trigger uses a 260-unit box. A native card holds the three swipe
rows. The 80-unit speed dial row centers the trigger and the selected caption.

## See also

- [First window](../README.md#first-window)
- [FPAS widget reference](../../widgets.md)
- [Gallery](../../../../examples/ui/gui/gallery/README.md)

## Source files

- `examples/ui/gui/gallery/Pages/Actions/`
  - [Commands.fpas](../../../../examples/ui/gui/gallery/Pages/Actions/Commands.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Actions/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Actions/Update.fpas)
  - [View.fpas](../../../../examples/ui/gui/gallery/Pages/Actions/View.fpas)
- `examples/ui/gui/gallery/Pages/Buttons/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Buttons/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Buttons/Update.fpas)
  - [View.fpas](../../../../examples/ui/gui/gallery/Pages/Buttons/View.fpas)
- `tests/Fixtures/gui-host/`
  - [ActionWidgetsContract.fpas](../../../../tests/Fixtures/gui-host/ActionWidgetsContract.fpas)
  - [ActionsContract.fpas](../../../../tests/Fixtures/gui-host/ActionsContract.fpas)
  - [ActionsPickersContract.fpas](../../../../tests/Fixtures/gui-host/ActionsPickersContract.fpas)
