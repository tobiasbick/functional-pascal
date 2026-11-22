# Segmented control

The **Components → Buttons → Segmented control** section shows the CUI
gallery's view switcher (**List**, **Board**, **Timeline**) and compact date
range selector (**Day**, **Week**, **Month**, **Year**).

Use `SegmentedControls.SegmentedControl` with a model-owned selected index.
Tab reaches the selected segment. Left and Right move to an adjacent segment
and propose selecting it. Home and End reach the ends. Click, Enter or Space
proposes the focused choice when it differs from the model.

The terminal shows the label and a selection marker for every segment.
`ControlSize` controls padding and row height. `FullWidth` distributes
available columns across equally sized segments. On a narrow terminal the
strip scrolls horizontally to keep the focused segment visible. Disabled
strips skip input. Use `SegmentedControls.ItemId(Id, Index)` to focus a segment
or find its `segment` snapshot.

## See also

- [Shared API](../../widgets.md#segmented-control)
- [GUI segmented control](../../gui/actions/segmented-control.md)

## Source files

- [SegmentedControl.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Actions/SegmentedControl.fpas)
- [ChoiceSamples.fpas](../../../../examples/ui/tui/gallery/ChoiceSamples.fpas)
