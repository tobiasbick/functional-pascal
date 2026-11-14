using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Pickers;

/// <summary>A color proposal and the native popup's visibility.</summary>
/// <param name="Value">Straight-alpha RGBA with red in the low byte.</param>
/// <param name="Open">Whether the native palette accepts input.</param>
public readonly record struct CuiColorPickerResult(uint Value, bool Open);
