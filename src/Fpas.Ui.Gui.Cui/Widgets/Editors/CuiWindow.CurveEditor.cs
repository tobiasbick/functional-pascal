// [Documentation](../../../../docs/ui/gui/editing-tools/curve-editor.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
    /// <param name="xFormat">Optional horizontal axis and readout format.</param>
    /// <param name="yFormat">Optional vertical axis and readout format.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiCurveEdit? CurveEditor(string id, CuiCurveKey[] keys, int capacity, int selected, float xMin, float xMax, float yMin, float yMax, string xLabel, string yLabel, float height, int widthKind, float width, float playhead, int flags, CuiChartFormat? xFormat = null, CuiChartFormat? yFormat = null)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckChartFormat(xFormat);
        CheckChartFormat(yFormat);
        if (capacity < keys.Length) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = new CuiCurveKey[capacity];
        keys.CopyTo(buffer, 0);
        int count = keys.Length;
        using var memory = new ChartMemory();
        var options = new NativeCurveOptions
        {
            XLabel = memory.Text(OptionalAi(xLabel)), YLabel = memory.Text(OptionalAi(yLabel)),
            XFormat = memory.Format(xFormat), YFormat = memory.Format(yFormat),
            XMin = xMin, XMax = xMax, YMin = yMin, YMax = yMax, Height = height, Width = width,
            Playhead = playhead, WidthKind = widthKind, Flags = flags
        };
        fixed (CuiCurveKey* data = buffer)
            return NativeMethods.CurveEditor(context, Stable(id), data, ref count, capacity, ref selected, out var result, in options) == 0
                ? null
                : new(buffer[..count], selected, result.Added, result.Removed,
                    result.Changed != 0, result.Hovered != 0, result.X, result.Y);
    }
}
