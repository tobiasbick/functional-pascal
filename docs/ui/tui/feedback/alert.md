# Alert

The **Components → Feedback** page shows the CUI gallery's four inline alerts
for an update, a payment, low storage and a failed build. The terminal marks the
title with a symbol in the theme's tone color and wraps the title and description
to the available width. `Icon` configures the graphical presentation.

Set `Dismissible` to add a **×** button. Click it or focus it and press Enter
or Space. The host hides the alert and calls the optional `OnDismiss` mapper
once. Leaving the view and returning creates a fresh alert. Use
`Alerts.DismissId(Id)` to address the button. `Enabled := false` leaves the
alert visible and disables its dismiss action.

## See also

- [Shared API](../../widgets.md#alerts-progress-and-spinners)
- [GUI alert](../../gui/feedback/alert.md)

## Source files

- [Alert.fpas](../../../../src/Fpas.Ui.Tui/Internal/Widgets/Feedback/Alert.fpas)
- [AlertSamples.fpas](../../../../examples/ui/tui/gallery/AlertSamples.fpas)
