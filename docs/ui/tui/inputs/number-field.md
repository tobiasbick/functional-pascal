# Number field

The **Components → Inputs → Pickers** section shows the CUI gallery's **Seats**
example, starting at 3 with a range of 0–99.

Use `NumberFields.NumberField` with a finite value, ordered bounds and an
`OnChange` mapper. The field proposes changes as soon as the draft parses as a
finite number, clamped to the inclusive bounds. Enter, Escape or moving focus
away restores the accepted value formatted with `DecimalPlaces`. The editor
uses a period as the decimal separator. An optional suffix displays units.

Press Up or Down to step, or hold Shift for ten steps. The **−** and **+**
buttons accept clicks, Enter and Space. They are disabled at their respective
bounds. Use `NumberFields.DecreaseId(Id)` and `NumberFields.IncreaseId(Id)` to
address them.

## See also

- [Shared API](../../widgets.md#number-field)
- [GUI number field](../../gui/inputs/number-field.md)

## Source files

- [NumberField.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Inputs/NumberField.fpas)
- [InputSamples.fpas](../../../../examples/ui/tui/gallery/InputSamples.fpas)
