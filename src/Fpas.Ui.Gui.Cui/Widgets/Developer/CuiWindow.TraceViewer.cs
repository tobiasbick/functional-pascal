// [Documentation](../../../../docs/ui/gui/developer-tools/trace-viewer.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Interop.Developer;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native TraceViewer widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Tracks">Tracks supplied to CUI.</param>
    /// <param name="Spans">Spans supplied to CUI.</param>
    /// <param name="Markers">Markers supplied to CUI.</param>
    /// <param name="Start">Start supplied to CUI.</param>
    /// <param name="Stop">Stop supplied to CUI.</param>
    /// <param name="Scroll">Scroll supplied to CUI.</param>
    /// <param name="MarkStart">Mark Start supplied to CUI.</param>
    /// <param name="MarkEnd">Mark End supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="RowHeight">Row Height supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Links">Native dependencies between sorted span indices.</param>
    /// <param name="Focus">Whether to request keyboard focus.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiTraceResult TraceViewer(string Id, CuiDevTrack[] Tracks, CuiDevSpan[] Spans, CuiDevMarker[] Markers, double Start, double Stop, float Scroll, double MarkStart, double MarkEnd, int Selection, float Height, float RowHeight, int WidthKind, float Width, bool Enabled, CuiTraceLink[]? Links = null, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        Links ??= [];
        var links = new NativeTraceLink[Links.Length];
        for (int i = 0; i < Links.Length; i++)
        {
            var link = Links[i];
            if ((uint)link.From >= (uint)Spans.Length || (uint)link.To >= (uint)Spans.Length ||
                link.Tone is < 0 or > 8 || !float.IsFinite(link.Width) || link.Width < 0)
                throw new ArgumentException("Invalid trace dependency.", nameof(Links));
            links[i] = new() { From = link.From, To = link.To, Tone = link.Tone,
                Width = link.Width, Dashed = link.Dashed ? (byte)1 : (byte)0 };
        }
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevTraceViewer(context, Stable(Id), PackTrack(memory, Tracks), Tracks.Length, PackSpan(memory, Spans), Spans.Length, PackMarker(memory, Markers), Markers.Length, Start, Stop, Scroll, MarkStart, MarkEnd, Selection, Height, RowHeight, WidthKind, Width, flags, out var result, memory.Copy<NativeTraceLink>(links), links.Length, out int hoveredLink);
        return new(result.ToManaged(), hoveredLink);
    }

    /// <summary>Reads CUI's span geometry from the most recently presented frame.</summary>
    /// <param name="id">The same identity supplied to TraceViewer.</param>
    /// <param name="span">A span in that trace; its name and color are not used for geometry.</param>
    /// <returns>Logical bounds, or an empty rectangle when absent, clipped out or folded.</returns>
    public CuiRectangle TraceSpanRectangle(string id, CuiDevSpan span)
    {
        CheckOpen();
        if (inFrame || !hasPresentedFrame)
            throw new InvalidOperationException("Present the current frame before reading trace geometry.");
        if (!double.IsFinite(span.Start) || !double.IsFinite(span.End) || span.End < span.Start ||
            span.Track < 0 || span.Depth < 0)
            throw new ArgumentException("Invalid trace span.", nameof(span));
        return NativeMethods.TraceSpanRectangle(context, Stable(id), span.Start, span.End, span.Track, span.Depth);
    }
}
