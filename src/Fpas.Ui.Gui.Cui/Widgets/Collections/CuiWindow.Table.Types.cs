using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>A native column descriptor.</summary>
/// <param name="Title">Literal heading.</param>
/// <param name="Kind">Encoded extent kind.</param>
/// <param name="Width">Extent value or grow weight.</param>
/// <param name="Align">Native text alignment.</param>
/// <param name="Sortable">Whether the header can be sorted.</param>
public readonly record struct CuiTableColumn(string Title, int Kind, float Width, int Align, bool Sortable);

/// <summary>The sort proposed by the native header.</summary>
/// <param name="Column">Zero-based column, minus one for unsorted, or minus two for no sorting.</param>
/// <param name="Descending">Whether the order is descending.</param>
public readonly record struct CuiTableSort(int Column, bool Descending);
