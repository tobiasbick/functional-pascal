using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>Native data for CanvasControls.</summary>
/// <param name="Icon">Icon supplied to CUI.</param>
/// <param name="Label">Label supplied to CUI.</param>
public sealed record CuiCanvasToggle(string Icon, string Label);
