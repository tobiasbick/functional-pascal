// [Documentation](../../../../docs/ui/gui/developer-tools/trace-viewer.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Interop.Developer;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Provides the order required by CUI's native trace viewer.</summary>
public static class CuiTraceOrder
{
    /// <summary>Sorts span indices by track, depth, start and descending end time.</summary>
    /// <param name="spans">The spans in their original order.</param>
    /// <returns>The old index of each span in sorted order.</returns>
    public static unsafe int[] Sort(CuiDevSpan[] spans)
    {
        ArgumentNullException.ThrowIfNull(spans);
        var native = new NativeDevSpan[spans.Length];
        for (int index = 0; index < native.Length; index++)
        {
            CuiDevSpan span = spans[index];
            if (!double.IsFinite(span.Start) || !double.IsFinite(span.End) || span.End < span.Start ||
                span.Track < 0 || span.Depth < 0)
                throw new ArgumentException("Invalid trace span.", nameof(spans));
            // CUI's comparator leaves the color key untouched. Carry the original index in it.
            native[index] = new() { Start = span.Start, End = span.End, Track = span.Track,
                Depth = span.Depth, ColorKey = (uint)index };
        }

        fixed (NativeDevSpan* items = native)
            NativeMethods.TraceSort(items, native.Length);

        var order = new int[native.Length];
        for (int index = 0; index < order.Length; index++)
            order[index] = (int)native[index].ColorKey;
        return order;
    }
}
