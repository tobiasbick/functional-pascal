// [Documentation](../../../../docs/ui/gui/editing-tools/flame-graph.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Reads span geometry from the most recently presented flame graph.</summary>
    /// <param name="id">The same stable identity supplied to FlameGraph.</param>
    /// <param name="span">A model span in that graph. Geometry uses its start, end and depth.</param>
    /// <returns>Logical bounds, or an empty rectangle outside the plot or for an absent graph.</returns>
    public CuiRectangle FlameSpanRectangle(string id, CuiFlameSpan span)
    {
        CheckPresentedGeometry();
        ArgumentNullException.ThrowIfNull(span);
        if (!double.IsFinite(span.Start) || !double.IsFinite(span.End) || span.End < span.Start || span.Depth < 0)
            throw new ArgumentException("Invalid flame graph span.", nameof(span));
        return NativeMethods.FlameSpanRectangle(context, Stable(id), span.Start, span.End, span.Depth);
    }

    /// <summary>Draws a native FlameGraph.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="spans">Ordered spans.</param>
    /// <param name="view">Model-owned state.</param>
    /// <param name="minimum">Minimum zoom range in seconds.</param>
    /// <param name="height">Logical height or native default.</param>
    /// <param name="rowHeight">Logical height per row.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="flags">Native adapter options.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiFlameRange? FlameGraph(string id, CuiFlameSpan[] spans, CuiFlameRange view, double minimum, float height, float rowHeight, int widthKind, float width, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        using var memory = new ChartMemory();
        var native = new NativeFlameSpan[spans.Length];
        for (int index = 0; index < native.Length; index++)
        {
            var item = spans[index];
            native[index] = new() { Name = memory.Text(item.Name), Start = item.Start, End = item.End, Depth = item.Depth, ColorKey = item.ColorKey };
        }
        var state = new NativeFlameRange { Start = view.Start, End = view.End, Scroll = view.Scroll, Selected = view.Selected, Zoomed = -1 };
        fixed (NativeFlameSpan* data = native)
            return NativeMethods.FlameGraph(context, Stable(id), data, native.Length, ref state, minimum, height, rowHeight, widthKind, width, flags) == 0
                ? null
                : new(state.Start, state.End, state.Scroll, state.Selected, state.Zoomed,
                    state.Hovered, state.ViewChanged != 0, state.SelectionChanged != 0);
    }
}
