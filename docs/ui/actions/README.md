# Actions

Buttons, command menus, and controlled selections from the FPAS gallery's
Buttons and Actions pages.

| Widget | When to use it | Hosts |
| --- | --- | --- |
| [Button](button.md) | Run a single action such as saving a document or opening a view. | GUI and TUI |
| [Icon button](icon-button.md) | Use a glyph for a compact, familiar action. Give it a tooltip that names the action. | GUI and TUI |
| [Button group](button-group.md) | Join related actions in a single horizontal frame with dividers. | GUI and TUI |
| [Copy button](copy-button.md) | Send literal text to the clipboard and show brief feedback next to the copied value. | GUI and TUI |
| [Hold to confirm](hold-button.md) | Require a deliberate hold before a destructive or hard-to-undo action. | GUI and TUI |
| [Segmented control](segmented-control.md) | Choose one view or filter from a compact row of alternatives. | GUI and TUI |
| [Dropdown menu](dropdown-menu.md) | Open a list of commands from a labeled or icon-only trigger. | GUI and TUI |
| [Split button](split-button.md) | Put the primary action and related commands on the same control. | GUI and TUI |
| [Toggle group](toggle-group.md) | Group independent flags such as text formatting or exclusive choices such as alignment. | GUI |
| [Confirm button](confirm-button.md) | Ask for a second activation before carrying out a destructive action. | GUI and TUI |
| [Status button](status-button.md) | Show the progress and outcome of work in the button that started it. | GUI and TUI |
| [Reactions](reactions.md) | Show counts and the current user's reaction alongside a message or activity. | GUI |
| [Theme switcher](theme-switcher.md) | Select Light, Dark or an application-provided System appearance. | GUI and TUI |
| [Theme toggle](theme-toggle.md) | Switch between two concrete appearances with a single button. | GUI and TUI |
| [User menu](user-menu.md) | Put account commands behind a trigger that shows the user name and initials. | GUI |
| [Swipe actions](swipe-actions.md) | Reveal actions next to a list row without taking up space in its resting layout. | GUI |
| [Speed dial](speed-dial.md) | Group closely related commands behind one floating action button. | GUI |

Use the same stable IDs across views. Widgets propose typed messages when
the user acts. Store accepted changes in the model and apply them in the
next view. In menu and button options, shortcut captions display a hint.

## See also

- [First window](../README.md#first-window)
- [FPAS widget reference](../../../src/Fpas.Ui/README.md)
- [Gallery](../../../examples/ui/gui/gallery/README.md)
