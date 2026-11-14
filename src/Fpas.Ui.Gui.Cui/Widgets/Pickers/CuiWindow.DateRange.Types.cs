using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Pickers;

/// <summary>A native date range selection and its visible parts.</summary>
/// <param name="Start">Start day number, or minus one.</param>
/// <param name="End">End day number, or minus one.</param>
/// <param name="Changed">Whether CUI proposed a changed selection.</param>
/// <param name="Open">Whether a picker popup is open.</param>
/// <param name="VisibleParts">Day numbers and codes for visible navigation and presets.</param>
public readonly record struct CuiDateRangeResult(int Start, int End, bool Changed, bool Open, int[] VisibleParts);
