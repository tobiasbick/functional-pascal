# Search field

The **Components → Inputs** page shows the CUI gallery's **Search projects**
example. The query is model-owned and limited to 63 UTF-8 bytes in the example.

Use `SearchFields.SearchField` with an `OnChange` mapper. The field supports
keyboard selection, paste, an optional Enter submission, and the same label,
placeholder, hint and error as a [text field](text-field.md). Prefix and suffix
text sit outside the editable value. The editor uses one terminal row.
`Icon` and `ControlSize` configure the graphical presentation.

For a nonempty editable query, click **×** or reach **Clear search** with Tab
and press Enter or Space. When the model accepts the empty query, the button
disappears and focus returns to the editor. `SearchFields.ClearId(Id)` addresses
the button. Read-only queries can be selected. Disabled controls skip input.

## See also

- [Shared API](../../widgets.md#search-field)
- [GUI search field](../../gui/inputs/search-field.md)

## Source files

- [SearchField.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Inputs/SearchField.fpas)
- [InputSamples.fpas](../../../../examples/ui/tui/gallery/InputSamples.fpas)
