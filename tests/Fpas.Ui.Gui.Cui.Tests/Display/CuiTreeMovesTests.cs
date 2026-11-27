// [Documentation](../../../docs/ui/gui/data-grid/tree-table.md)
using Fpas.Ui.Gui.Cui.Widgets.Collections;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks native tree moves and model index remapping without a window.</summary>
public sealed class CuiTreeMovesTests
{
    /// <summary>Moves two subtrees before, into, or after a target in their supplied order.</summary>
    [Theory]
    [InlineData(0, new int[] { 1, 0, 2, 3, 4 }, new int[] { -1, -1, 1, -1, 3 })]
    [InlineData(1, new int[] { 3, 4, 1, 0, 2 }, new int[] { -1, 0, 0, 0, 3 })]
    [InlineData(2, new int[] { 3, 4, 1, 0, 2 }, new int[] { -1, 0, -1, -1, 3 })]
    public void MovesSubtreesAndUpdatesParentIndices(int placement, int[] order, int[] newParents)
    {
        int[] parents = [-1, -1, 0, -1, 3];
        CuiTreeMove result = CuiTreeMoves.Move(parents, [1, 0], 3, placement);
        Assert.Equal(order, result.Order);
        Assert.Equal(newParents, result.Parents);
        for (int index = 0; index < order.Length; index++)
            Assert.Equal(index, result.IndexMap[order[index]]);
        Assert.Equal([-1, -1, 0, -1, 3], parents);
    }

    /// <summary>Preserves model order and indices when the target is in a moved subtree.</summary>
    [Fact]
    public void RefusedMoveKeepsOriginalIndices()
    {
        int[] parents = [-1, -1, 0];
        CuiTreeMove result = CuiTreeMoves.Move(parents, [0], 2, 1);
        Assert.Equal([0, 1, 2], result.Order);
        Assert.Equal(parents, result.Parents);
        Assert.Equal([0, 1, 2], result.IndexMap);
    }
}
