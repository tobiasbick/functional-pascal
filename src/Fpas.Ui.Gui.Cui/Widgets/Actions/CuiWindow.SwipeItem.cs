// [Documentation](../../../../docs/ui/gui/actions/swipe-actions.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins CUI's swipe row and its content scope.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="labels">Optional action labels in left then right order.</param>
    /// <param name="icons">Optional action glyphs in the same order.</param>
    /// <param name="tones">Native action tone ordinals.</param>
    /// <param name="leftCount">Number of actions on the left.</param>
    /// <param name="enabled">Whether gestures and row input are accepted.</param>
    /// <param name="icon">Optional leading icon.</param>
    /// <param name="trackAction">Maps combined action indices to identities, or null.</param>
    /// <returns>Tracked action indices in left then right order.</returns>
    public unsafe int[] BeginSwipeItem(string id, string?[] labels, string?[] icons, int[] tones,
        int leftCount, bool enabled, string icon, Func<int, string>? trackAction)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        ArgumentNullException.ThrowIfNull(tones);
        ArgumentOutOfRangeException.ThrowIfNegative(leftCount);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(leftCount, labels.Length);
        if (leftCount > 100 || labels.Length != icons.Length || labels.Length != tones.Length)
            throw new ArgumentException("Supply matching action arrays with no more than 100 left actions.");
        Utf8.Validate(icon, nameof(icon));
        for (int i = 0; i < labels.Length; i++)
        {
            if (labels[i] is { } label) Utf8.Validate(label, nameof(labels));
            if (icons[i] is { } glyph) Utf8.Validate(glyph, nameof(icons));
            ArgumentOutOfRangeException.ThrowIfNegative(tones[i]);
            ArgumentOutOfRangeException.ThrowIfGreaterThan(tones[i], 8);
        }
        int[] nodes = new int[labels.Length + 1];
        int node;
        fixed (int* colors = tones, parts = nodes)
            node = NativeMethods.SwipeBegin(context, Stable(id), labels, icons, colors, leftCount,
                labels.Length - leftCount, enabled ? 1 : 0, Optional(icon), parts);
        if (node < 0) throw new InvalidOperationException("CUI's swipe row nesting limit was reached.");
        layouts.Add(new(Layout.SwipeItem, Node: node));
        if (trackAction is null) return [];
        TrackControl(id, node);
        List<int> visible = [];
        for (int i = 0; i < labels.Length; i++)
        {
            if (nodes[i + 1] < 0) continue;
            TrackControl(trackAction(i), nodes[i + 1]);
            visible.Add(i);
        }
        return visible.ToArray();
    }

    /// <summary>Closes the native row and reads its chosen action.</summary>
    /// <param name="detail">Optional trailing annotation.</param>
    /// <returns>Left index, right index plus 100, or minus one.</returns>
    public int EndSwipeItem(string detail)
    {
        CheckFrame();
        Utf8.Validate(detail, nameof(detail));
        var scope = RequireLayout(Layout.SwipeItem);
        int result = NativeMethods.SwipeEnd(context, scope.Node, Optional(detail));
        layouts.RemoveAt(layouts.Count - 1);
        return result;
    }
}
