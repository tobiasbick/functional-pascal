using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Pickers;

/// <summary>A proposed time and the native choices currently tracked in the popup.</summary>
/// <param name="Value">The time at minute precision.</param>
/// <param name="Choices">Native hour, minute and period codes.</param>
public readonly record struct CuiTimePickerResult(TimeOnly Value, int[] Choices);
