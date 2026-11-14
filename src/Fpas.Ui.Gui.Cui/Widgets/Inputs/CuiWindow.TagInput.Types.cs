using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Model proposal and visible remove actions for a native tag input.</summary>
/// <param name="Tags">Ordered tags after the frame's edit, or the original array if unchanged.</param>
/// <param name="DisplayValue">Current tags joined with comma-space separators.</param>
/// <param name="RemoveCount">Number of remove buttons drawn for the input tags.</param>
public readonly record struct CuiTagInputResult(string[] Tags, string DisplayValue, int RemoveCount);
