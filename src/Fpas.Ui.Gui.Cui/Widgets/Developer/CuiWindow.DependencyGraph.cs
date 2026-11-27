// [Documentation](../../../../docs/ui/gui/developer-tools/dependency-graph.md)
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
    /// <param name="Selection">Model selection, minus one for none, or null for native selection.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="TopDown">Top Down supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to request focus on the graph.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult DependencyGraph(string Id, CuiDevGraphNode[] Vertices, CuiDevEdge[] Edges,
        int? Selection, float Height, bool TopDown, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        CheckFeedbackNumber(Height, nameof(Height));
        if (Selection is int vertex && (vertex < -1 || vertex >= Vertices.Length))
            throw new ArgumentOutOfRangeException(nameof(Selection));
        foreach (var node in Vertices)
        {
            Utf8.Validate(node.Label, nameof(Vertices));
            if (node.Detail is string detail) Utf8.Validate(detail, nameof(Vertices));
            if ((uint)node.Tone > 8) throw new ArgumentOutOfRangeException(nameof(Vertices));
        }
        foreach (var edge in Edges)
            if ((uint)edge.From >= Vertices.Length || (uint)edge.To >= Vertices.Length)
                throw new ArgumentOutOfRangeException(nameof(Edges));
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevDependencyGraph(context, Stable(Id), PackGraphNode(memory, Vertices), Vertices.Length,
            PackEdge(memory, Edges), Edges.Length, Selection ?? -2, Height, TopDown ? 1 : 0, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
