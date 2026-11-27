# User menu

Put account commands behind a trigger that shows the user name and initials. Available in **GUI**.

![User menu in the FPAS gallery](images/user-menu.png)

## Example

```pascal
unit Docs.Ui.Actions.UserMenu;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Overlays.Menus as M;
uses Fpas.Ui.Widgets.Overlays.UserMenus as U;

// Builds the example view.
// ## Returns
// The widget with its message type.
public function View(): E.Element of string;
begin
  return U.UserMenu of string('account', 'Maya Okafor', [
    (M.Entry of string).Action(new M.Item of string('settings', 'Settings', 'settings')),
    (M.Entry of string).Action(new M.Item of string('logout', 'Log out', 'logout'))
  ], new U.Options(Detail := 'maya@example.com'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Detail` | Null omits detail from the trigger and popup header. An empty string reserves a blank line. |
| `Status` | Null omits the avatar status dot. `online`, `busy` and `away` use different colors. Other strings, including the empty string, use gray. |
| `AvatarPath` | Null uses initials. A PNG/JPEG path supplies the native trigger and popup image. Failed loads use initials. |
| `Compact` | false. Set true to show only the avatar in the trigger. |
| `Enabled` | true. |
| `Position` | `BottomStart` selects CUI's default. The full trigger uses `TopStart` and the compact trigger uses `BottomEnd`. |
| `Width` | Zero uses the native 248-unit popup width. |

## Interaction

Click or press Enter or Space on the trigger. Press Up or Down to highlight
a command. Select an enabled command to send its message and close the popup.
Press Escape or click outside to dismiss the popup. The user menu uses the
same keyed commands and decorations as a [dropdown menu](dropdown-menu.md).

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_user_menu)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Overlays/UserMenus.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Actions/Widgets/UserMenu.fpas)
- [Actions](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Overlays/`
  - [UserMenu.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Overlays/UserMenu.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Overlays/`
  - [NativeMethods.UserMenu.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Overlays/NativeMethods.UserMenu.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Overlays/`
  - [CuiWindow.UserMenu.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Overlays/CuiWindow.UserMenu.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/overlays/`
  - [user_menu.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/user_menu.c)
  - [user_menu.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/overlays/user_menu.h)
