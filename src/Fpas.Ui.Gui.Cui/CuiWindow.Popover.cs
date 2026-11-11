using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins a native popover anchored to the last button.</summary>
    /// <param name="id">Stable panel and snapshot identity.</param>
    /// <param name="placement">CUI placement relative to the anchor.</param>
    /// <param name="width">Logical width, or zero to fit.</param>
    /// <param name="padding">Logical inset, or zero for the native default.</param>
    /// <param name="gap">Logical spacing, or zero for the native default.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <param name="enabled">Whether the panel accepts input.</param>
    /// <returns>Draw the body for bit 1, propose dismissal for bit 2 and expose an open panel for bit 4.</returns>
    public int BeginPopover(string id, int placement, float width, float padding, float gap, bool open, bool enabled)
    {
        CheckFrame();
        if (placement is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(placement));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (!float.IsFinite(padding) || padding < 0) throw new ArgumentOutOfRangeException(nameof(padding));
        if (!float.IsFinite(gap) || gap < 0) throw new ArgumentOutOfRangeException(nameof(gap));
        CheckPopupDepth();
        int anchor = NativeMethods.LastNode(context);
        int result = NativeMethods.PopoverBegin(context, Stable(id), placement, width, padding, gap,
            open ? 1 : 0, enabled ? 1 : 0, out int node, out uint owner);
        if (owner != 0) TrackModal(id, owner);
        if ((result & 1) != 0) layouts.Add(new(Layout.Popover, Node: anchor));
        if (node < 0) return result;
        trackedNodes.Add(id, node);
        return result | 4;
    }

    /// <summary>Closes the drawing scope and restores the anchor's layout target.</summary>
    public void EndPopover()
    {
        var scope = RequireLayout(Layout.Popover);
        NativeMethods.PopoverEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }

    private void CheckPopupDepth()
    {
        if (layouts.Count(static scope => scope.Kind == Layout.Popover) >= 15)
            throw new InvalidOperationException("CUI supports up to sixteen nested popup scopes.");
    }
}
