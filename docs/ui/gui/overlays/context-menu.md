# Context menu

Attach a context menu to the native control that should receive a right click. Available in **GUI**.

![Context menu in the FPAS gallery](images/context-menu.png)

## Example

```pascal
unit Docs.Ui.Overlays.ContextMenu;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Menus as W;

// Builds the context menu example.
// ## Parameters
// - `Target`: The control that receives the overlay.
// ## Returns
// The widget for the application view.
public function View(Target: E.Element of string): E.Element of string;
begin
  return W.ContextMenu of string('context', Target, [
    (W.Entry of string).Action(new W.Item of string('copy', 'Copy', 'copy', Shortcut := 'Ctrl C')),
    (W.Entry of string).Separator,
    (W.Entry of string).Action(new W.Item of string('delete', 'Delete', 'delete', Danger := true))
  ]);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Id` | A stable menu ID. |
| `Target` | The element that owns the right-click area. |
| `Entries` | Ordered commands, section captions and separators. See [command options](../actions/dropdown-menu.md#options-and-defaults). |
| `Enabled` | true. The target retains its own input settings. |

## Interaction

Right-click the target to open the menu at the pointer. Press Up or Down to
move the highlight. Press Enter on an enabled command to choose it and close
the menu. Press Escape or click outside to dismiss the menu. CUI owns the open
state.

Interactive descendants keep their own hit areas. Attach the menu to the
element that receives the intended right click. CUI uses the command's shortcut
text as a hint. Bind the shortcut separately if it also runs a command.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_context_menu)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/Menus.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Overlays/Widgets/ContextMenu.fpas)
- [Dropdown menu](../actions/dropdown-menu.md)
- [Overlays](README.md)

## Source files

- [ContextMenu.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/ContextMenu.fpas)
- [ContextContract.fpas](../../../../tests/Fixtures/gui-host/ContextContract.fpas)
- [CuiContextMenuTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Overlays/CuiContextMenuTests.cs)
