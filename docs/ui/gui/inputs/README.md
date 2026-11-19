# Inputs

The Inputs gallery page combines text editors, credentials and form fields.
Keep each editable value in the model and apply proposals in the update
function. The single-line text field also runs in the TUI. The other input
widgets on this page require the GUI host.

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
| [Calendar](calendar.md) | Keep a single-date calendar visible. |

## See also

- [UI handbook](../README.md)
- [Smart inputs](../smart-inputs/README.md)
- [Pickers](../pickers/README.md)
- [Inputs in the gallery](../../../../examples/ui/gui/gallery/Pages/Inputs/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Inputs/`
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Update.fpas)
- `examples/ui/gui/gallery/Pages/Inputs/Widgets/`
  - [CredentialFields.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/CredentialFields.fpas)
  - [PickerFields.fpas](../../../../examples/ui/gui/gallery/Pages/Inputs/Widgets/PickerFields.fpas)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [input_regressions.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/input_regressions.c)
- `tests/Fixtures/gui-host/`
  - [CredentialFieldsContract.fpas](../../../../tests/Fixtures/gui-host/CredentialFieldsContract.fpas)
- `tests/Fpas.Ui.Gui.Cui.Tests/Inputs/`
  - [CuiOptionalTrackingTests.cs](../../../../tests/Fpas.Ui.Gui.Cui.Tests/Inputs/CuiOptionalTrackingTests.cs)
