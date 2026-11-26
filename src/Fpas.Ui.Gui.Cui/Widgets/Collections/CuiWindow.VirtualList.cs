// [Documentation](../../../../docs/ui/gui/data-display/virtual-list.md)
using Fpas.Ui.Gui.Cui.Widgets.Collections;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens CUI's virtual viewport and returns its half-open row range.</summary>
    /// <param name="id">Stable list identity.</param>
    /// <param name="count">Total number of rows.</param>
    /// <param name="rowHeight">Fixed height of each row in logical points.</param>
    /// <param name="heightKind">Encoded viewport extent kind.</param>
    /// <param name="height">Logical viewport extent or percentage.</param>
    /// <param name="enabled">Whether the viewport accepts wheel input.</param>
    /// <param name="restoreOffset">Logical offset to restore, or -1 to keep the native position.</param>
    /// <returns>Indices CUI needs for this frame.</returns>
    /// <exception cref="InvalidOperationException">Opening this list would exceed a native CUI nesting limit.</exception>
    public CuiVirtualRange BeginVirtualList(string id, int count, float rowHeight, int heightKind,
        float height, bool enabled, float restoreOffset = -1)
    {
        CheckFrame();
        CheckExtent(heightKind, height, nameof(height));
        ArgumentOutOfRangeException.ThrowIfNegative(count);
        if (!float.IsFinite(rowHeight) || rowHeight < 1 || (double)rowHeight * count >= int.MaxValue)
            throw new ArgumentOutOfRangeException(nameof(rowHeight));
        if (!float.IsFinite(restoreOffset) || (restoreOffset < 0 && restoreOffset != -1))
            throw new ArgumentOutOfRangeException(nameof(restoreOffset));
        int node = NativeMethods.VirtualListBegin(context, Stable(id), count, rowHeight, heightKind,
            height, enabled ? 1 : 0, restoreOffset, out int first, out int last);
        if (node < 0) throw new InvalidOperationException("The native CUI virtual list nesting limit was reached.");
        layouts.Add(new(Layout.VirtualList, Node: node));
        return new(first, last);
    }

    /// <summary>Scopes a realized row's native identities by its logical index.</summary>
    /// <param name="index">Logical row index.</param>
    public void BeginVirtualRow(int index)
    {
        RequireLayout(Layout.VirtualList);
        NativeMethods.VirtualRowBegin(context, index);
        layouts.Add(new(Layout.VirtualRow));
    }

    /// <summary>Closes the innermost virtual row identity scope.</summary>
    public void EndVirtualRow()
    {
        RequireLayout(Layout.VirtualRow);
        NativeMethods.VirtualRowEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the native virtual viewport.</summary>
    public void EndVirtualList()
    {
        var scope = RequireLayout(Layout.VirtualList);
        NativeMethods.VirtualListEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
