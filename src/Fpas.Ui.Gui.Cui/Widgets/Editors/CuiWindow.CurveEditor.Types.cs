// [Documentation](../../../../docs/ui/gui/editing-tools/curve-editor.md)
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
/// <param name="Changed">Whether an edit changed a key.</param>
/// <param name="Hovered">Whether the pointer is over the plot.</param>
/// <param name="HoverX">Horizontal axis coordinate under the pointer.</param>
/// <param name="HoverY">Curve value at that horizontal coordinate.</param>
public sealed record CuiCurveEdit(CuiCurveKey[] Keys, int Selected, int Added, int Removed,
    bool Changed, bool Hovered, float HoverX, float HoverY);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCurveOptions
{
    internal byte* XLabel, YLabel;
    internal float XMin, XMax, YMin, YMax, Height, Width, Playhead;
    internal int WidthKind, Flags;
    internal byte* XFormat, YFormat;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeCurveObservation
{
    internal int Added, Removed, Changed, Hovered;
    internal float X, Y;
}
