using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's ranked bars; native label-keyed animation survives reordering.</summary>
    /// <param name="id">Stable list identity.</param>
    /// <param name="items">Nonnegative amounts with unique literal labels.</param>
    /// <param name="options">Native sorting, row limit, color and format.</param>
    /// <param name="enabled">Allow pointer input.</param>
    /// <param name="track">Track inspection and retirement identity.</param>
    /// <returns>Clicked original input index, otherwise minus one.</returns>
    public unsafe int BarList(string id, CuiVizItem[] items, CuiBarListOptions options, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckBarList(items, options);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.BarList(context, stable, memory.Items(items), items.Length, options.WidthKind, options.Width,
            memory.Format(options.Format), options.MaximumRows, options.KeepOrder ? 1 : 0, options.Color.Palette, options.Color.Rgba,
            enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node); return clicked;
    }
}
