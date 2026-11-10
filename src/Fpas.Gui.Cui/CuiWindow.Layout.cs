using System.Runtime.InteropServices;
using Fpas.Gui.Cui.Interop;

namespace Fpas.Gui.Cui;

public sealed partial class CuiWindow
{
    [StructLayout(LayoutKind.Sequential)]
    internal struct FlexNode
    {
        internal int Node, Horizontal, Kind;
        internal float Basis, Minimum, Maximum, Grow, Shrink;
    }
    private readonly Dictionary<int, FlexNode> flexNodes = [];

    private unsafe void ResolveHostLayout()
    {
        if (flexNodes.Count == 0) return;
        FlexNode[] nodes = flexNodes.Values.OrderBy(node => node.Node).ToArray();
        fixed (FlexNode* items = nodes) NativeMethods.ResolveLayout(context, items, nodes.Length);
        foreach (var (id, request) in scrollRequests)
            if (scrollNodes.TryGetValue(id, out int node)) NativeMethods.ScrollLayout(context, node, request.Mode, request.Offset);
        if (scrollRequests.Count != 0) NativeMethods.ResolveLayout(context, null, 0);
    }
}
