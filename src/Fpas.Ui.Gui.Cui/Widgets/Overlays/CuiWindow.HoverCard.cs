using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins CUI's hover card at the preceding interactive control.</summary>
    /// <param name="id">Stable panel identity.</param>
    /// <param name="placement">Preferred CUI placement.</param>
    /// <param name="width">Logical width, or zero for the native default.</param>
    /// <param name="delay">Hover delay in seconds, or zero for the native default.</param>
    /// <param name="enabled">Whether the card may open.</param>
    /// <returns>Bit 1 for an active drawing scope, and bit 2 for an open panel.</returns>
    public int BeginHoverCard(string id, int placement, float width, float delay, bool enabled)
    {
        CheckFrame();
        if (placement is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(placement));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (!float.IsFinite(delay) || delay < 0) throw new ArgumentOutOfRangeException(nameof(delay));
        CheckPopupDepth();
        int anchor = NativeMethods.LastNode(context);
        int draw = NativeMethods.HoverCardBegin(context, Stable(id), placement, width, delay,
            enabled ? 1 : 0, out int node, out uint owner);
        if (owner != 0) TrackModal(id, owner);
        if (draw != 0) layouts.Add(new(Layout.HoverCard, Node: anchor));
        if (node < 0) return draw;
        trackedNodes.Add(id, node);
        return draw | 2;
    }

    /// <summary>Closes the native scope and restores its trigger for layout.</summary>
    public void EndHoverCard()
    {
        var scope = RequireLayout(Layout.HoverCard);
        NativeMethods.HoverCardEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
