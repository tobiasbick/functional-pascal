using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's grouped or stacked bar chart.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="series">Named finite sample arrays.</param>
    /// <param name="options">Native chart settings.</param>
    /// <param name="track">Retain the public inspection identity.</param>
    public unsafe void BarChart(string id, CuiChartSeries[] series, CuiChartOptions options, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options);
        CheckChartSeries(series, options, true);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int node = NativeMethods.BarChart(context, stable, memory.Series(series), series.Length, in native);
        if (track) TrackControl(id, node);
    }
}
