using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's squarified treemap and optional controlled selection.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="items">Named nonnegative amounts with a finite sum.</param>
    /// <param name="options">Native dimensions and formatting.</param>
    /// <param name="selected">Null disables selection; minus one means no selected item.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Native activation and selection proposal.</returns>
    public unsafe CuiTreemapResult Treemap(string id, CuiVizItem[] items, CuiTreemapOptions options,
        int? selected = null, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckVizItems(items);
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckVizDistance(options.Height); CheckChartFormat(options.Format);
        int proposal = selected ?? -1;
        if (proposal < -1 || proposal >= items.Length) throw new ArgumentOutOfRangeException(nameof(selected));
        using var memory = new ChartMemory();
        int clicked = NativeMethods.Treemap(context, stable, memory.Items(items), items.Length,
            options.WidthKind, options.Width, options.Height, memory.Format(options.Format),
            selected.HasValue ? 1 : 0, ref proposal, enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return new(clicked, proposal);
    }
}
