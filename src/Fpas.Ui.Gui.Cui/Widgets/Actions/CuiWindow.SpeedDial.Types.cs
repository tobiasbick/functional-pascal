using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Actions;

/// <summary>A native speed dial's choice and visible action rows.</summary>
/// <param name="Choice">Chosen action index, or minus one.</param>
/// <param name="Open">Whether the dial accepts action input.</param>
/// <param name="VisibleActions">Original indices of tracked action rows.</param>
public readonly record struct CuiSpeedDialResult(int Choice, bool Open, int[] VisibleActions);
