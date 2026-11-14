using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>A native-compatible curve key.</summary>
/// <param name="X">Horizontal coordinate.</param>
/// <param name="Y">Value.</param>
/// <param name="InX">Incoming X offset.</param>
/// <param name="InY">Incoming Y offset.</param>
/// <param name="OutX">Outgoing X offset.</param>
/// <param name="OutY">Outgoing Y offset.</param>
/// <param name="Mode">Interpolation mode.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiCurveKey(float X, float Y, float InX, float InY, float OutX, float OutY, byte Mode);

/// <summary>Result of a native curve edit.</summary>
/// <param name="Keys">Proposed curve.</param>
/// <param name="Selected">Selected index.</param>
/// <param name="Added">Inserted index.</param>
/// <param name="Removed">Removed index.</param>
public sealed record CuiCurveEdit(CuiCurveKey[] Keys, int Selected, int Added, int Removed);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCurveOptions
{
    internal byte* XLabel, YLabel;
    internal float XMin, XMax, YMin, YMax, Height, Width, Playhead;
    internal int WidthKind, Flags;
}
