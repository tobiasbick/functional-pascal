# Tabs

Use tabs to switch the content within one page. The gallery shows an underline strip and a pill strip with icons. Available in **GUI**.

![Tabs in the FPAS gallery](images/tabs.png)

## Example

```pascal
unit Docs.Ui.Navigation.Tabs;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Navigation.Tabs as C;

// Builds the tabs example.
// ## Parameters
// - `Selected`: Current value from the application model.
// ## Returns
// The widget for the application view.
public function View(Selected: int): E.Element<int>;
begin
  return C.Tabs<int>('tabs', [new C.Item('Overview'), new C.Item('Reports')],
    Selected, function(Index: int): int => Index);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Items` | One through 32 tabs. Icons are optional. |
| `Selected` | Zero-based index from the model. |
| `Appearance` | Underline. Pill is also available. |
| `FullWidth / Enabled` | false / true. |

## Interaction

Click a tab or move with Left and Right while the strip has focus. The callback receives the proposed index. Use it to select the content in your view. CUI scrolls overflowing strips horizontally. Use `ItemId(Id, Index)` to focus a specific tab.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Navigation/Tabs.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Navigation/Widgets/Tabs.fpas)
- [Sidebar navigation](sidebar.md)
- [Navigation](README.md)
