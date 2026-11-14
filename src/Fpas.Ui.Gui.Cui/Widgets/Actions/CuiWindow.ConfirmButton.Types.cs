using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Actions;

/// <summary>Reports activation and the native confirmation state after drawing.</summary>
/// <param name="Confirmed">Whether this activation completed the confirmation.</param>
/// <param name="Armed">Whether the control is waiting for a second activation.</param>
public readonly record struct CuiConfirmButtonResult(bool Confirmed, bool Armed);
