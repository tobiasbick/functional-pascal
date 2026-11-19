# Sidebar navigation

Use a sidebar for named destinations grouped under optional headings. Available in **GUI and TUI**.

![Sidebar navigation in the FPAS gallery](images/sidebar.png)

## Example

```pascal
unit Docs.Ui.Navigation.Sidebar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Navigation.Navigation as C;

// Builds the sidebar navigation example.
// ## Parameters
// - `Active`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Active: string?): E.Element<string>;
begin
  return C.Sidebar<string>('navigation', [
    new C.Section<string>('Workspace', [
      new C.Item<string>('overview', 'Overview', 'overview'),
      new C.Item<string>('inbox', 'Inbox', 'inbox', Badge := '3')
    ])
  ], Active);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `ActiveKey` | A destination key or null for no active destination. |
| `Title / GapBefore` | Use an empty title to omit the heading. GapBefore defaults to 0. Positive spacing requires GUI. |
| `Icon / Badge` | Both empty by default. GUI supports CUI icons. Both hosts show badges. |
| `Enabled` | true for the sidebar and for each item. |

## Interaction

Click a destination or activate it with Enter or Space while it has focus. The sidebar sends the item's message. Update the active key and displayed page together. Keys are unique across the sidebar. Use `ItemId(Id, Key)` for focus. Place the sidebar in a scroll view when its sections exceed the available height. In the TUI, Up/Down move focus
within the sidebar. Page Up/Page Down move by a page and Home/End move to the
first or last enabled destination. The scroll view reveals the focused row.

## Appearance controls

The gallery places a segmented Light and Dark control and eight canvas
swatches under the destinations. The appearance and accent belong to
the shared model and drive both the sidebar and the narrow drawer.

![Light appearance and neutral accent in the gallery sidebar](images/sidebar-appearance.png)

![Dark appearance and blue accent in the gallery sidebar](images/sidebar-appearance-dark.png)

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Navigation.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/App/Sidebar.fpas)
- [Tabs](tabs.md)
- [Segmented control](../actions/segmented-control.md)
- [Interactive canvas](../text-and-display/canvas.md)
- [Theme selection and palette](../actions/theme-switcher.md)
- [Navigation](README.md)
- [Terminal example](../../tui/navigation/sidebar.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Navigation/`
  - [Sidebar.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Navigation/Sidebar.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Navigation/`
  - [NativeMethods.Navigation.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Navigation/NativeMethods.Navigation.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Navigation/`
  - [CuiWindow.Navigation.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Navigation/CuiWindow.Navigation.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/navigation/`
  - [sidebar.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/sidebar.c)
  - [sidebar.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/navigation/sidebar.h)
