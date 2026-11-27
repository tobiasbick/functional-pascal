# Spinner

The **Components → Feedback** page pairs a spinner with **Syncing 24 files**.

Use `Spinners.Spinner` while work is in progress and the amount remaining is
unknown. Pair it with a short activity label. The terminal uses one cell and
chooses its glyph sequence from `Animation`. Choose `Color` for explicit ink
or `ThemeColor` for a palette token. The terminal spinner defaults to the theme
accent. The graphical host uses the requested size and track color.

Visible spinners advance automatically in an interactive session, including
while the application waits for input. Headless frames and themes with
`Motion := 0.0F` use the first glyph. A spinner has no focus stop or input.

## See also

- [Shared API](../../widgets.md#alerts-progress-and-spinners)
- [GUI spinner](../../gui/feedback/spinner.md)

## Source files

- [Spinner.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Feedback/Spinner.fpas)
- [PlotSamples.fpas](../../../../examples/ui/tui/gallery/PlotSamples.fpas)
