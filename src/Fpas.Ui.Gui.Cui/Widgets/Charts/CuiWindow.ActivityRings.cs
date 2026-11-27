// [Documentation](../../../../docs/ui/gui/more-charts/activity-rings.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws up to eight native activity rings, including values above one.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="items">Named nonnegative progress fractions.</param>
    /// <param name="options">Native size, thickness and captions.</param>
    /// <param name="enabled">Allow pointer input.</param>
    /// <param name="track">Track inspection identity.</param>
    /// <returns>Clicked ring input index, otherwise minus one.</returns>
    public unsafe int ActivityRings(string id, CuiVizItem[] items, CuiActivityRingsOptions options, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckVizItems(items);
        if (items.Length > 8) throw new ArgumentOutOfRangeException(nameof(items));
        CheckVizDistance(options.Size); CheckVizDistance(options.Thickness);
        if (options.Center is { } center) Utf8.Validate(center, nameof(options));
        if (options.Caption is { } caption) Utf8.Validate(caption, nameof(options));
        using var memory = new ChartMemory();
        int clicked = NativeMethods.ActivityRings(context, stable, memory.Items(items), items.Length, options.Size,
            options.Thickness, memory.Text(options.Center), memory.Text(options.Caption), options.Legend ? 1 : 0,
            enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node); return clicked;
    }
}
