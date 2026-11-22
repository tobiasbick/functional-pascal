# Sidebar navigation

Group destinations under headings. In the gallery, the selected destination determines which terminal example appears in the right panel.

![Scrollable Components and Extended navigation beside the selected preview](../../../../examples/ui/tui/gallery/panels.png)

## Example

```pascal
unit Docs.Tui.Widgets.Sidebar;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Navigation.Navigation as W;

// Builds the sidebar navigation example.
// ## Parameters
// - `Active`: Key of the selected destination.
// ## Returns
// The resulting view.
public function View(Active: string): E.Element<string>;
begin
  return E.ScrollView<string>('pages-scroll', W.Sidebar<string>('pages', [
    new W.Section<string>('Library', [
      new W.Item<string>('tracks', 'Tracks', 'tracks'), new W.Item<string>('albums', 'Albums', 'albums')
    ])], Active));
end function;

end unit;
```

## Interaction

Up/Down moves focus without changing the page. Enter, Space or a click sends the destination message. Apply the active key and page together in Update. Home/End skips to the first or last enabled item. Page Up/Page Down and the wheel navigate longer lists.

## See also

- [Navigation](README.md)
- [Shared API](../../widgets.md#sidebar-navigation)
- [GUI sidebar navigation](../../gui/navigation/sidebar.md)
- [Gallery source](../../../../examples/ui/tui/gallery/main.fpas)

## Source files

- [RowNavigation.fpas](../../../../src/Fpas.Ui.Tui/Internal/Hosting/RowNavigation.fpas)
- [Sidebar.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Navigation/Sidebar.fpas)
