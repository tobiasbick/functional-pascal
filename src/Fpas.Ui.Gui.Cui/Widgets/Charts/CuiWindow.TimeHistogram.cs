// [Documentation](../../../../docs/ui/gui/more-developer-tools/time-histogram.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native TimeHistogram widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Start">Start supplied to CUI.</param>
    /// <param name="Bucket">Bucket supplied to CUI.</param>
    /// <param name="Buckets">Buckets supplied to CUI.</param>
    /// <param name="Series">Series supplied to CUI.</param>
    /// <param name="RangeStart">Range Start supplied to CUI.</param>
    /// <param name="RangeEnd">Range End supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="Utc">Utc supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>The proposed endpoints, selection change flag and hovered bucket.</returns>
    public unsafe CuiDevResult TimeHistogram(string Id, double Start, double Bucket, int Buckets, CuiDevSeries[] Series, double RangeStart, double RangeEnd, float Height, bool Utc, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Buckets < 0)
            throw new ArgumentOutOfRangeException(nameof(Buckets));
        if (!double.IsFinite(Start) || !double.IsFinite(Bucket) || Bucket <= 0 || !double.IsFinite(Start + Bucket * Buckets))
            throw new ArgumentException("The histogram requires a finite time window and positive bucket duration.", nameof(Bucket));
        if (!double.IsFinite(RangeStart) || !double.IsFinite(RangeEnd))
            throw new ArgumentException("Selection endpoints must be finite.", nameof(RangeStart));
        if (!float.IsFinite(Height) || Height < 0)
            throw new ArgumentOutOfRangeException(nameof(Height));
        foreach (var series in Series)
        {
            if (series.Tone is < 0 or > 8)
                throw new ArgumentException("A series tone must be a native CUI tone.", nameof(Series));
            if (series.Counts is { } counts && (counts.Length != Buckets || counts.Any(count => !float.IsFinite(count) || count < 0)))
                throw new ArgumentException("A series must specify finite nonnegative counts for every bucket, or omit all counts.", nameof(Series));
        }
        using var memory = new ChartMemory();
        NativeMethods.DevTimeHistogram(context, Stable(Id), Start, Bucket, Buckets, PackSeries(memory, Series), Series.Length, RangeStart, RangeEnd, Height, Utc ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
