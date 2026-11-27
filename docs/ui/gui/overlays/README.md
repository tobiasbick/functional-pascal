# Overlays

Dialogs, sheets, guided tours and the command palette take their visibility
from the model.
Context menus and tooltips open through native pointer interaction. Return
a toast command from your update to send a notification.

| Widget | When to use it | Hosts |
| --- | --- | --- |
| [Dialog](dialog.md) | Block interaction with the underlying view for a short task. | GUI and TUI |
| [Action footer](action-footer.md) | Align a dialog's actions in the native footer. | GUI |
| [Sheet and drawer](sheet.md) | Open a modal panel at a window edge. | GUI and TUI |
| [Command palette](command-palette.md) | Search for application commands and destinations. | GUI |
| [Context menu](context-menu.md) | Offer commands on right click. | GUI |
| [Popover](popover.md) | Open an anchored panel, such as a filter form. | GUI |
| [Tooltip](tooltip.md) | Show brief pointer help and a shortcut hint. | GUI |
| [Toast](toast.md) | Show a notification without moving focus. | GUI |
| [Guided tour](guided-tour.md) | Highlight controls and explain them in order. | GUI |

## Overlays gallery page

The page groups triggers in four [native cards](../feedback/card.md).
The Dialogs and sheets card opens Edit profile, Notifications, Delete project
and the command palette. The profile dialog shares Name and Email with Inputs.
Its Public profile switch shares a value with Wi-Fi on Selection.

The Popover and context menu card uses a gap of 16 logical units. Filters
opens a 280-unit panel anchored to the button. The checkboxes and slider
share their values with Email notifications, Marketing and Brightness on
Selection. Apply closes the panel. The muted context target opens Cut,
Copy, Paste and Delete on right click.

The Toasts card shows four notifications. The archive notification includes
an Undo action. Hover over the stack to expand it and pause expiration.
The Tooltips card uses a [wrapping row](../layout/row.md) with a gap of 4.
A [divider](../layout/divider.md) separates formatting from alignment, list
and link controls. The first three tools include shortcut hints.

## See also

- [UI handbook](../README.md)
- [Disclosure](../disclosure/README.md)

## Source files

- [State.fpas](../../../../examples/ui/gui/gallery/Pages/Overlays/State.fpas)
- [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Overlays/Update.fpas)
- [View.fpas](../../../../examples/ui/gui/gallery/Pages/Overlays/View.fpas)
- [CuiWidgetOptionTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiWidgetOptionTests.cs)
