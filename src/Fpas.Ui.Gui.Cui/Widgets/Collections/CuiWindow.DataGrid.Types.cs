using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>Native grid column descriptor.</summary>
/// <param name="Title">Literal heading.</param>
/// <param name="Width">Initial width, or zero for the native default.</param>
/// <param name="Alignment">Shared table text alignment.</param>
/// <param name="Sortable">Whether to propose sorting.</param>
/// <param name="Fixed">Whether to suppress resizing.</param>
/// <param name="Priority">Responsive hiding priority, with zero always visible.</param>
public readonly record struct CuiGridColumn(string Title, float Width, int Alignment, bool Sortable, bool Fixed, int Priority);

/// <summary>Grid viewport and proposed model state.</summary>
/// <param name="First">First realized row.</param>
/// <param name="Last">Exclusive end of the realized range.</param>
/// <param name="Selected">Writable one-byte mask per row, or null.</param>
/// <param name="State">Native sort, descending, cursor and anchor.</param>
/// <param name="Headers">Whether each native header was realized.</param>
public readonly record struct CuiGridFrame(int First, int Last, byte[]? Selected, int[] State, bool[] Headers);
