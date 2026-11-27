# Inputs

Keep each editable value in the model and apply proposals in the update
function. The [terminal input guide](../../tui/inputs/README.md) lists the
widgets available in the TUI.

| Widget | When to use it |
| --- | --- |
| [Text field](text-field.md) | Edit a single line with an optional label, hint or validation error. |
| [Search field](search-field.md) | Enter and clear a query. |
| [Text area](text-area.md) | Edit wrapping multiline text. |
| [Tag input](tag-input.md) | Enter and remove free-form tags. |
| [One-time code input](one-time-code.md) | Enter a short verification code with a completion callback. |
| [Password strength meter](password-strength.md) | Show a strength estimate below a password field. |
| [Select](select.md) | Choose one item from a list with optional search. |
| [Number field](number-field.md) | Type or step through a bounded numeric value. |
| [Date picker](date-picker.md) | Choose one date from a form field. |
| [Color picker](color-picker.md) | Edit a swatch with a palette and presets. |

## Inputs gallery page

The three sections use [native cards](../feedback/card.md) with a gap of
16 logical units. Text fields and Pickers use a [responsive grid](../feedback/grid.md).
Each grid fits up to two columns with a minimum width of 260 logical units
and a gap of 16. About spans the first card.

The page uses [columns](../layout/column.md) to group Verification code,
New password and Brand color with related content. Password and New password
share a value. The first field shows validation. The second has a strength
meter. Completing the verification code shows the Code entered toast.

The shared gallery shell uses [rows](../layout/row.md) for the sidebar,
compact header and centered page area. The Inputs cards use grids and columns.

## See also

- [UI handbook](../README.md)
- [Smart inputs](../smart-inputs/README.md)
- [Pickers](../pickers/README.md)
- [CUI Inputs comparison](../../../future/cui-compare/README.md#inputs-evidence)

## Source files

- `examples/ui/gui/gallery/Pages/Inputs/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Update.fpas)
  - [View.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/View.fpas)
- `examples/ui/gui/gallery/Pages/Inputs/Widgets/`
  - [CredentialFields.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/CredentialFields.fpas)
  - [PickerFields.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/PickerFields.fpas)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [input_regressions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/input_regressions.c)
- `tests/Fixtures/gui-host/`
  - [CredentialFieldsContract.fpas](../../../../tests/Fixtures/gui-host/CredentialFieldsContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiOptionalTrackingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiOptionalTrackingTests.cs)
