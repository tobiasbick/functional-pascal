using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>Frame output from the native tree table.</summary>
/// <param name="Activated">Activated node, or minus one.</param>
/// <param name="Load">Node indices requesting children.</param>
public readonly record struct CuiTreeTableEvents(int Activated, int[] Load);
