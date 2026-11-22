# Table

Use `Fpas.Ui.Widgets.Collections.Tables.Table` for rows with stable IDs and plain text cells. In **Components → Data display → Table**, the terminal gallery shows Name, Role and Status. Select a person to read their email address and revenue below the table.

## Example

```pascal
unit Docs.Tui.Widgets.Table;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Collections.Tables as T;

// Builds a table that sends the selected row ID to the application.
// ## Parameters
// - `Selected`: Currently selected ID.
// ## Returns
// A two-column table.
public function View(Selected: string): E.Element<string>;
begin
  return T.Table<string>('members', [new T.Column('Name'), new T.Column('Role')], [
    new T.Row<string>('priya', [T.Cell<string>.Text('Priya Nair'), T.Cell<string>.Text('Admin')], Selected = 'priya')
  ], null, null, function(Id: string): string => Id);
end function;

end unit;
```

## Interaction

Use ↑/↓, Home/End or Page Up/Page Down on a focused row. Enter, Space or a click proposes selection by ID. Tab to a sortable heading and press Enter, or click it, to request a new order. The application owns the order and selection. In the gallery, → moves from the sidebar to the selected person and Escape returns to the sidebar.

Each row occupies one terminal line. Columns support fixed, content and weighted fill widths, plus left, center and right alignment. Long cells clip at grapheme boundaries. A bounded table keeps its header in place while the wheel or row navigation scrolls the body. Use `Cell.Text` for terminal data. Composed cells such as avatars and badges require the GUI.

Set `Tables.Options.HorizontalScroll := true` to retain natural widths on a
narrow terminal. With a body row focused, Ctrl+Left and Ctrl+Right reveal
columns. The header pans with the body. Arrows without modifiers keep their
vertical navigation behavior.

## See also

- [Data display](README.md)
- [GUI table and shared options](../../gui/data-display/table.md)
- [Gallery](../../../../examples/ui/tui/gallery/README.md)

## Source files

- [Terminal table layout and interactions](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Collections/Table.fpas)
- [Terminal gallery example](../../../../examples/ui/tui/gallery/PeopleSample.fpas)
