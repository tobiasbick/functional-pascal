using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's responsive donut and share legend.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="values">Nonnegative shares with a finite sum.</param>
    /// <param name="labels">One caption per share, or an empty array for no legend.</param>
    /// <param name="widthKind">Native bridge width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="height">Maximum diameter; zero uses the native default.</param>
    /// <param name="format">Optional safe tooltip number formatting.</param>
    /// <param name="legend">Show the native share legend.</param>
    /// <param name="track">Retain the public inspection identity.</param>
    public unsafe void DonutChart(string id, float[] values, string[] labels, int widthKind = 0, float width = 0,
        float height = 0, CuiChartFormat? format = null, bool legend = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); CheckChartValues(values);
        ArgumentNullException.ThrowIfNull(labels);
        if (labels.Length != 0 && labels.Length != values.Length)
            throw new ArgumentException("Supply one label per share or no labels.", nameof(labels));
        var options = new CuiChartOptions(widthKind, width, height, labels, format, Legend: legend);
        CheckChartOptions(options);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int node = NativeMethods.DonutChart(context, stable, memory.Copy<float>(values), values.Length, in native);
        if (track) TrackControl(id, node);
    }
}
