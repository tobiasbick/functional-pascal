using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>The native picker proposal and visible options.</summary>
/// <param name="Selection">Fresh selection flags on a change, or null.</param>
/// <param name="Open">Whether the popup accepts input.</param>
/// <param name="VisibleOptions">Original indices of tracked option rows.</param>
public readonly record struct CuiMultiSelectResult(bool[]? Selection, bool Open, int[] VisibleOptions);
