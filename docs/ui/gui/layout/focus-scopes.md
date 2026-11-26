# Focus scopes

Use a focus scope when Tab and Shift+Tab must stay inside the active pane.
The GUI host supports focus scopes on rows, columns, layered stacks and grids.

1. Give each pane a stable ID in `Layout.FocusScope`.
2. Assign the scope to `Layout.Flow.Scope`. For a grid, set
   `Grids.Options.Flow` to the flow record with the scope.
3. Set `Active` from the model when the view builds the pane.
4. Update the active pane in response to a message.

Tab and Shift+Tab visit focusable controls inside the active scope.
Controls in inactive panes still accept pointer input when enabled.
An inactive scope also excludes all nested scopes from Tab navigation.
If several scopes are active, CUI uses the last active scope submitted in
the frame.
If the view has no active scope, Tab visits unscoped controls.
An open modal limits Tab navigation to its own controls and overlays.

`FocusScope.Id` names the pane for keyboard navigation.
Use `Flow.Id` if the application also needs the pane bounds in its snapshot.
Use a control ID with the `Mvu.Focus` host request to set focus explicitly.

## See also

- [CUI API coverage](../../../cui-api-coverage.md#application-capabilities)
- [Container options](container-options.md)
- [Layout inspection](layout-inspection.md)
- [Canvas keyboard input](../text-and-display/canvas.md)
- [GUI host](../host.md#frame-preparation-and-presentation)
- [GUI layout](README.md)

## Source files

- [Layout.fpas](../../../../src/Fpas.Ui/Layout.fpas)
- [Focus scope presentation](../../../../src/Fpas.Ui.Gui/Internal/Presentation/Presentation.fpas)
- [Grid focus scope presentation](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Containers/Grid.fpas)
- [Focus contract tests](../../../../tests/Fixtures/gui-host/DesktopFocusContract.fpas)
