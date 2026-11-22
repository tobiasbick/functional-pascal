# Spinner

The **Components → Feedback** page shows the CUI sizes, activity styles, and
the inline examples **Saving…**, **Syncing 24 files**, and **Connecting**.

Use `Spinners.Spinner` while work is in progress and the amount remaining is
unknown. The terminal rounds the logical diameter to columns and rows. Each
style has a sequence of terminal glyphs. `Color` and `Track` use the shared
chart palette or explicit colors, and `Inverse` changes the background and
default ink.

Visible spinners advance automatically in an interactive session, including
while the application waits for input. Headless frames and themes with
`Motion := 0.0F` use the first glyph. A spinner has no focus stop or input.

## See also

- [Shared API](../../widgets.md#alerts-progress-and-spinners)
- [GUI spinner](../../gui/feedback/spinner.md)

## Source files

- [Spinner.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Feedback/Spinner.fpas)
- [PlotSamples.fpas](../../../../examples/ui/tui/gallery/PlotSamples.fpas)
