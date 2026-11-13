using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native matrix heatmap.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="values">Finite samples in row-major order.</param>
    /// <param name="rows">Nonnegative row count.</param>
    /// <param name="columns">Nonnegative column count.</param>
    /// <param name="options">Native labels, color and scale.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Clicked row-major input index, otherwise minus one.</returns>
    public unsafe int Heatmap(string id, float[] values, int rows, int columns, CuiHeatmapOptions options,
        bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(values); ArgumentNullException.ThrowIfNull(options);
        CheckHeatmap(values, rows, columns, options);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.Heatmap(context, stable, memory.Copy<float>(values), rows, columns,
            memory.Labels(options.RowLabels), memory.Labels(options.ColumnLabels), options.Cell, options.Minimum,
            options.Maximum, options.Color.Palette, options.Color.Rgba, memory.Format(options.Format),
            options.WidthKind, options.Width, enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return clicked;
    }
}
