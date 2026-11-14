using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Pickers;

/// <summary>A proposed single date and its visible native calendar parts.</summary>
/// <param name="Value">The selected date, or null before a selection.</param>
/// <param name="VisibleParts">Day numbers, navigation (-8/-9), title (-10), months (-100-year*12-month+1),
/// and years (-200000-year).</param>
public readonly record struct CuiCalendarResult(DateOnly? Value, int[] VisibleParts);
