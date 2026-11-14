using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native flows and preserves original input indices when zero flows are hidden.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="nodes">Named native nodes.</param>
    /// <param name="links">Directed nonnegative flows.</param>
    /// <param name="options">Native dimensions and format.</param>
    /// <param name="enabled">Allow pointer input.</param>
    /// <param name="track">Track inspection identity.</param>
    /// <returns>Clicked node index, node count plus original link index, or minus one.</returns>
    public unsafe int Sankey(string id, CuiSankeyNode[] nodes, CuiSankeyLink[] links, CuiSankeyOptions options,
        bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckSankey(nodes, links, options);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.Sankey(context, stable, memory.Nodes(nodes), nodes.Length, memory.Copy<CuiSankeyLink>(links),
            links.Length, options.WidthKind, options.Width, options.Height, options.NodeWidth, memory.Format(options.Format),
            enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node); return clicked;
    }
}
