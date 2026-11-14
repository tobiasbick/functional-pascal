using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Native commit proposal and retained edit status.</summary>
/// <param name="Value">Confirmed proposal, otherwise the original model value.</param>
/// <param name="Draft">Current native draft, including uncommitted text.</param>
/// <param name="Editing">Whether native inline editing remains active.</param>
public sealed record CuiInlineEditResult(string Value, string Draft, bool Editing);
