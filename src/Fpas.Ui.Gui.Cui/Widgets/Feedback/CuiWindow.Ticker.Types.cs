using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Feedback;

/// <summary>A native ticker entry.</summary>
/// <param name="Label">Visible label.</param>
/// <param name="Value">Optional secondary value.</param>
/// <param name="Delta">Percent change, with zero hiding the trend.</param>
/// <param name="LowerIsBetter">Whether a fall is favorable.</param>
public readonly record struct CuiTickerItem(string Label, string Value, float Delta, bool LowerIsBetter);
