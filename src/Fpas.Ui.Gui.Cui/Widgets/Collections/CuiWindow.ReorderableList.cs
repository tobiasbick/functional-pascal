// [Documentation](../../../../docs/ui/gui/data-display/reorderable-list.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private sealed record ReorderList(int Count, bool Enabled);

    /// <summary>Opens CUI's drag-to-reorder list with a frame-owned copy of the model order.</summary>
    /// <param name="id">Stable list identity.</param>
    /// <param name="order">Current permutation of canonical item indices.</param>
    /// <param name="enabled">Whether a native drag can change the order.</param>
    /// <returns>The proposed order for drawing this frame's items.</returns>
    public unsafe int[] BeginReorder(string id, int[] order, bool enabled)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(order);
        if (order.Length is < 1 or > 256) throw new ArgumentOutOfRangeException(nameof(order));
        Span<bool> seen = stackalloc bool[order.Length];
        seen.Clear();
        foreach (int index in order)
        {
            if ((uint)index >= (uint)order.Length || seen[index])
                throw new ArgumentException("Order must be a permutation.", nameof(order));
            seen[index] = true;
        }
        var proposed = new int[order.Length];
        int node;
        fixed (int* input = order, output = proposed)
            node = NativeMethods.ReorderBegin(context, Stable(id), input, order.Length, enabled ? 1 : 0, output);
        layouts.Add(new(Layout.Reorder, Node: node, Data: new ReorderList(order.Length, enabled)));
        return proposed;
    }

    /// <summary>Opens a row in the current proposed display order.</summary>
    /// <param name="position">Zero-based position in the proposed order.</param>
    /// <param name="enabled">Whether the row's handle accepts input.</param>
    public void BeginReorderItem(int position, bool enabled)
    {
        var list = (ReorderList)RequireLayout(Layout.Reorder).Data!;
        if ((uint)position >= (uint)list.Count) throw new ArgumentOutOfRangeException(nameof(position));
        int node = NativeMethods.ReorderItemBegin(context, position, enabled && list.Enabled ? 1 : 0);
        layouts.Add(new(Layout.ReorderItem, Node: node));
    }

    /// <summary>Closes the innermost native reorder row.</summary>
    public void EndReorderItem()
    {
        var scope = RequireLayout(Layout.ReorderItem);
        NativeMethods.ReorderItemEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the list and reports whether a drag changed the order.</summary>
    /// <returns>Whether CUI changed the order this frame.</returns>
    public bool EndReorder()
    {
        var scope = RequireLayout(Layout.Reorder);
        bool changed = NativeMethods.ReorderEnd(context, scope.Node) != 0;
        layouts.RemoveAt(layouts.Count - 1);
        return changed;
    }
}
