using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Actions;

/// <summary>The result of drawing a native split button.</summary>
/// <param name="Choice">Minus one for no action, zero for the main action, or one plus the menu index.</param>
/// <param name="Open">Whether the popup currently accepts input.</param>
/// <param name="Highlighted">The highlighted menu index, or minus one.</param>
public readonly record struct CuiSplitButtonResult(int Choice, bool Open, int Highlighted);
