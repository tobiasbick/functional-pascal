using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>A detached native signature snapshot.</summary>
/// <param name="Coordinates">Interleaved X and Y coordinates in CUI's logical drawing space.</param>
/// <param name="StrokeEnds">Exclusive end point index of each stroke.</param>
public readonly record struct CuiSignature(float[] Coordinates, int[] StrokeEnds);
