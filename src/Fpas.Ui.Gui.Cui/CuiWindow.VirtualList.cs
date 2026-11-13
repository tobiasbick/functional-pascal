using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>The rows CUI wants to realize, including overscan.</summary>
/// <param name="First">First inclusive row.</param>
/// <param name="Last">Last exclusive row.</param>
public readonly record struct CuiVirtualRange(int First, int Last);

public sealed partial class CuiWindow
{
    /// <summary>Opens CUI's virtual viewport and returns its half-open row range.</summary>
    /// <param name="id">Stable list identity.</param>
    /// <param name="count">Total number of rows.</param>
    /// <param name="rowHeight">Fixed height of each row in logical points.</param>
    /// <param name="heightKind">Encoded viewport extent kind.</param>
    /// <param name="height">Logical viewport extent or percentage.</param>
    /// <param name="enabled">Whether the viewport accepts wheel input.</param>
    /// <returns>Indices CUI needs for this frame.</returns>
    public CuiVirtualRange BeginVirtualList(string id, int count, float rowHeight, int heightKind,
        float height, bool enabled)
    {
        CheckFrame();
        CheckExtent(heightKind, height, nameof(height));
        ArgumentOutOfRangeException.ThrowIfNegative(count);
        if (!float.IsFinite(rowHeight) || rowHeight < 1 || (double)rowHeight * count >= int.MaxValue)
            throw new ArgumentOutOfRangeException(nameof(rowHeight));
        int node = NativeMethods.VirtualListBegin(context, Stable(id), count, rowHeight, heightKind,
            height, enabled ? 1 : 0, out int first, out int last);
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
