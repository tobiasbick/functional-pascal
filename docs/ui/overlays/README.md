# Overlays

Dialogs, sheets and the command palette take their visibility from the model.
Context menus and tooltips open through native pointer interaction. Return
a toast command from your update to send a notification.

| Widget | When to use it | Hosts |
| --- | --- | --- |
| [Dialog](dialog.md) | Block interaction with the underlying view for a short task. | GUI |
| [Action footer](action-footer.md) | Align a dialog's actions in the native footer. | GUI |
| [Sheet and drawer](sheet.md) | Open a modal panel at a window edge. | GUI and TUI |
| [Command palette](command-palette.md) | Search for application commands and destinations. | GUI |
| [Context menu](context-menu.md) | Offer commands on right click. | GUI |
| [Popover](popover.md) | Open an anchored panel, such as a filter form. | GUI |
| [Tooltip](tooltip.md) | Show brief pointer help and a shortcut hint. | GUI |
| [Toast](toast.md) | Show a notification without moving focus. | GUI |

## See also

- [UI handbook](../README.md)
- [Disclosure](../disclosure/README.md)
- [Overlays in the gallery](../../../examples/ui/gui/gallery/Pages/Overlays/View.fpas)
