using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's calendar heatmap, resolving cropped-week clicks to the original input.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="start">Date corresponding to the first input sample.</param>
    /// <param name="values">Finite daily samples; nonpositive values are empty.</param>
    /// <param name="options">Native cells, palette, format and unit.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Clicked original input index, otherwise minus one.</returns>
    public unsafe int CalendarHeatmap(string id, DateOnly start, float[] values, CuiCalendarHeatmapOptions options,
        bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options);
        CheckVizSamples(values, false); CheckVizDistance(options.Cell); CheckChartColor(options.Color);
        CheckChartFormat(options.Format); Utf8.Validate(options.Unit, nameof(options));
        if (values.Length > 0) _ = start.AddDays(values.Length - 1);
        float total = 0;
        foreach (float value in values) total += Math.Max(0, value);
        if (!float.IsFinite(total)) throw new ArgumentOutOfRangeException(nameof(values));
        using var memory = new ChartMemory();
        int clicked = NativeMethods.CalendarHeatmap(context, stable, start.Year, start.Month, start.Day,
            memory.Copy<float>(values), values.Length, options.Cell, options.Color.Palette, options.Color.Rgba,
            memory.Format(options.Format), memory.Text(options.Unit), enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return clicked;
    }
}
