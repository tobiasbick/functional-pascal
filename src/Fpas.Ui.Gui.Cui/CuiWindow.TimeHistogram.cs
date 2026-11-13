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
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult TimeHistogram(string Id, double Start, double Bucket, int Buckets, CuiDevSeries[] Series, double RangeStart, double RangeEnd, float Height, bool Utc, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Buckets < 0 || Series.Any(series => series.Counts.Length != Buckets))
            throw new ArgumentException("Every series must contain one count per bucket.", nameof(Series));
        using var memory = new ChartMemory();
        NativeMethods.DevTimeHistogram(context, Stable(Id), Start, Bucket, Buckets, PackSeries(memory, Series), Series.Length, RangeStart, RangeEnd, Height, Utc ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
