using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CurveEditor.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="keys">Ordered keys.</param>
    /// <param name="capacity">Maximum item count.</param>
    /// <param name="selected">Selected index.</param>
    /// <param name="xMin">Horizontal minimum.</param>
    /// <param name="xMax">Horizontal maximum.</param>
    /// <param name="yMin">Vertical minimum.</param>
    /// <param name="yMax">Vertical maximum.</param>
    /// <param name="xLabel">Horizontal readout label.</param>
    /// <param name="yLabel">Vertical readout label.</param>
    /// <param name="height">Logical height or native default.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="playhead">Playhead position.</param>
    /// <param name="flags">Native adapter options.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiCurveEdit? CurveEditor(string id, CuiCurveKey[] keys, int capacity, int selected, float xMin, float xMax, float yMin, float yMax, string xLabel, string yLabel, float height, int widthKind, float width, float playhead, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (capacity < keys.Length) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = new CuiCurveKey[capacity];
        keys.CopyTo(buffer, 0);
        int count = keys.Length;
        using var memory = new ChartMemory();
        var options = new NativeCurveOptions
        {
            XLabel = memory.Text(OptionalAi(xLabel)), YLabel = memory.Text(OptionalAi(yLabel)),
            XMin = xMin, XMax = xMax, YMin = yMin, YMax = yMax, Height = height, Width = width,
            Playhead = playhead, WidthKind = widthKind, Flags = flags
        };
        fixed (CuiCurveKey* data = buffer)
            return NativeMethods.CurveEditor(context, Stable(id), data, ref count, capacity, ref selected, out int added, out int removed, in options) == 0
                ? null
                : new(buffer[..count], selected, added, removed);
    }
}
