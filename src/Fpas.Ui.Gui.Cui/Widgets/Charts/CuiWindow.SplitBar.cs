using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's segmented bar and wrapping legend.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="items">Named nonnegative amounts with a finite sum.</param>
    /// <param name="options">Native presentation.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Clicked part index, otherwise minus one.</returns>
    public unsafe int SplitBar(string id, CuiVizItem[] items, CuiSplitBarOptions options, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckVizItems(items);
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckVizDistance(options.Height); CheckChartFormat(options.Format);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.SplitBar(context, stable, memory.Items(items), items.Length,
            options.WidthKind, options.Width, options.Height, memory.Format(options.Format), options.Legend ? 1 : 0,
            enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return clicked;
    }
}
