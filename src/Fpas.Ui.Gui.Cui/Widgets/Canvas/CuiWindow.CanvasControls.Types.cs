// [Documentation](../../../../docs/ui/gui/canvas-and-bars/canvas-controls.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>Native data for CanvasControls.</summary>
/// <param name="Icon">CUI icon glyph. Null uses a circle.</param>
/// <param name="Label">Tooltip caption. Null omits the tooltip.</param>
public sealed record CuiCanvasToggle(string? Icon = null, string? Label = null);
