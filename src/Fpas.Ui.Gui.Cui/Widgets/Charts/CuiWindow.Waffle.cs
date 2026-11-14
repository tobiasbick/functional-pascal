using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's waffle squares and responsive legend.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="items">Named nonnegative amounts with a finite sum.</param>
    /// <param name="options">Native presentation.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Clicked item index, otherwise minus one.</returns>
    public unsafe int Waffle(string id, CuiVizItem[] items, CuiWaffleOptions options, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckVizItems(items);
        CheckVizDistance(options.Cell); CheckChartFormat(options.Format);
        if (options.Columns is < 1 or > 60 || options.Rows is < 1 or > 60)
            throw new ArgumentOutOfRangeException(nameof(options));
        using var memory = new ChartMemory();
        int clicked = NativeMethods.Waffle(context, stable, memory.Items(items), items.Length, options.Cell,
            options.Columns, options.Rows, memory.Format(options.Format), enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return clicked;
    }
}
