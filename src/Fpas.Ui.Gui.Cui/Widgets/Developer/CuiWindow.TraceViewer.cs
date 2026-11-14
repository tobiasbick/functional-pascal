using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

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
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult TraceViewer(string Id, CuiDevTrack[] Tracks, CuiDevSpan[] Spans, CuiDevMarker[] Markers, double Start, double Stop, float Scroll, double MarkStart, double MarkEnd, int Selection, float Height, float RowHeight, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevTraceViewer(context, Stable(Id), PackTrack(memory, Tracks), Tracks.Length, PackSpan(memory, Spans), Spans.Length, PackMarker(memory, Markers), Markers.Length, Start, Stop, Scroll, MarkStart, MarkEnd, Selection, Height, RowHeight, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
