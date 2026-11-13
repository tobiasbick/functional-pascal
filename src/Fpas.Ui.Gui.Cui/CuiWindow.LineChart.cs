using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's line/area chart with native hover, legend and animation.</summary>
    /// <param name="id">Stable identity independent of series names.</param>
    /// <param name="series">Named finite sample arrays.</param>
    /// <param name="options">Native chart settings.</param>
    /// <param name="track">Retain the public inspection identity.</param>
    public unsafe void LineChart(string id, CuiChartSeries[] series, CuiChartOptions options, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options);
        CheckChartSeries(series, options, false);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int node = NativeMethods.LineChart(context, stable, memory.Series(series), series.Length, in native);
        if (track) TrackControl(id, node);
    }
}
