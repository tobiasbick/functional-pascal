using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native DependencyGraph widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Vertices">Vertices supplied to CUI.</param>
    /// <param name="Edges">Edges supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="TopDown">Top Down supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult DependencyGraph(string Id, CuiDevGraphNode[] Vertices, CuiDevEdge[] Edges, int Selection, float Height, bool TopDown, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevDependencyGraph(context, Stable(Id), PackGraphNode(memory, Vertices), Vertices.Length, PackEdge(memory, Edges), Edges.Length, Selection, Height, TopDown ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
