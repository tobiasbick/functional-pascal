using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Selection and rows rendered by the native select popup.</summary>
/// <param name="Selected">Resulting positional index, or minus one.</param>
/// <param name="Open">Whether the option list is open.</param>
/// <param name="VisibleOptions">Indices of the rendered option rows.</param>
public readonly record struct CuiSelectResult(int Selected, bool Open, int[] VisibleOptions);
