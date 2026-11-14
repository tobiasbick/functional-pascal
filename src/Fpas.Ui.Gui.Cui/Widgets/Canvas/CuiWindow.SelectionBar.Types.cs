using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>Native data for SelectionBar.</summary>
/// <param name="Icon">Icon supplied to CUI.</param>
/// <param name="Label">Label supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
public sealed record CuiSelectionAction(string Icon, string Label, int Tone);
