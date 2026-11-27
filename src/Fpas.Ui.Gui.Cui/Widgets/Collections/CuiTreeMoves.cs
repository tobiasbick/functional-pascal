// [Documentation](../../../../docs/ui/gui/data-grid/tree-table.md)
using Fpas.Ui.Gui.Cui.Interop;
using static Fpas.Ui.Gui.Cui.Interop.NativeMethods;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>A relinked tree flattened to parent-before-child order.</summary>
/// <param name="Order">Old node index for each new position.</param>
/// <param name="Parents">New parent index for each new position.</param>
/// <param name="IndexMap">New position of each old node.</param>
public sealed record CuiTreeMove(int[] Order, int[] Parents, int[] IndexMap);

/// <summary>Relinks a model tree through CUI without creating a window.</summary>
public static class CuiTreeMoves
{
    /// <summary>Applies a native move and returns the array order and index mapping.</summary>
    /// <param name="parents">Parent index for each node, or minus one for a root.</param>
    /// <param name="moved">Node indices in the order they must move.</param>
    /// <param name="target">Destination node index.</param>
    /// <param name="placement">Encoded placement relative to the target.</param>
    /// <returns>The new order, parents, and old-to-new index mapping.</returns>
    public static unsafe CuiTreeMove Move(int[] parents, int[] moved, int target, int placement)
    {
        ArgumentNullException.ThrowIfNull(parents);
        ArgumentNullException.ThrowIfNull(moved);
        int count = parents.Length;
        var heads = new int[count + 1];
        Array.Fill(heads, -1);
        var original = new NativeTreeNode[count];
        for (int index = count - 1; index >= 0; index--)
        {
            int parent = parents[index];
            if (parent < -1 || parent >= index)
                throw new ArgumentException("Parents must precede their children.", nameof(parents));
            original[index] = new(parent, heads[index + 1], heads[parent + 1], 0);
            heads[parent + 1] = index;
        }

        var proposed = (NativeTreeNode[])original.Clone();
        int root = heads[0];
        fixed (NativeTreeNode* nodes = proposed)
        fixed (int* indexes = moved)
            root = NativeMethods.TreeMove(nodes, count, root, indexes, moved.Length, target, placement);

        var order = new int[count];
        var indexMap = new int[count];
        if (root == heads[0] && original.AsSpan().SequenceEqual(proposed))
        {
            for (int index = 0; index < count; index++) order[index] = indexMap[index] = index;
            return new(order, (int[])parents.Clone(), indexMap);
        }

        int oldIndex = root;
        var newParents = new int[count];
        for (int index = 0; index < count; index++)
        {
            NativeTreeNode node = proposed[oldIndex];
            order[index] = oldIndex;
            indexMap[oldIndex] = index;
            newParents[index] = node.Parent < 0 ? -1 : indexMap[node.Parent];
            if (node.FirstChild >= 0)
            {
                oldIndex = node.FirstChild;
            }
            else
            {
                while (oldIndex >= 0 && proposed[oldIndex].NextSibling < 0)
                    oldIndex = proposed[oldIndex].Parent;
                if (oldIndex >= 0) oldIndex = proposed[oldIndex].NextSibling;
            }
        }
        return new(order, newParents, indexMap);
    }
}
