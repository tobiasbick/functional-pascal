using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One model-owned native reaction.</summary>
/// <param name="Icon">CUI icon glyph.</param>
/// <param name="Count">Current nonnegative count.</param>
/// <param name="Active">Whether the current user has reacted.</param>
public readonly record struct CuiReaction(string Icon, int Count, bool Active);

/// <summary>A reaction change proposed by CUI.</summary>
/// <param name="Index">Zero-based changed item, or minus one.</param>
/// <param name="Count">Proposed count when an item changed.</param>
/// <param name="Active">Proposed activation when an item changed.</param>
public readonly record struct CuiReactionChange(int Index, int Count, bool Active);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's reaction group without modifying the application values.</summary>
    /// <param name="id">Stable group identity.</param>
    /// <param name="items">Nonempty list of ordered reactions.</param>
    /// <param name="enabled">Whether the group accepts input.</param>
    /// <param name="focus">Zero or one plus the requested item index.</param>
    /// <param name="trackingIds">Group and ordered item identities, or null.</param>
    /// <returns>The changed item proposed by CUI.</returns>
    public unsafe CuiReactionChange Reactions(string id, CuiReaction[] items, bool enabled, int focus,
        string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(items);
        int length = items.Length;
        if (length == 0) throw new ArgumentException("Supply at least one reaction.", nameof(items));
        if (focus < 0 || focus > length) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != length + 1)
            throw new ArgumentException("Supply group and item tracking IDs.", nameof(trackingIds));
        var icons = new string[length];
        var counts = new int[length];
        var active = new int[length];
        var nodes = new int[length + 1];
        for (int i = 0; i < length; i++)
        {
            CuiReaction item = items[i];
            Utf8.Validate(item.Icon, nameof(items));
            if (item.Icon.Length == 0 || item.Count < 0 || (!item.Active && item.Count == int.MaxValue))
                throw new ArgumentException("Supply an icon and an incrementable count for each inactive reaction.", nameof(items));
            icons[i] = item.Icon;
            counts[i] = item.Count;
            active[i] = item.Active ? 1 : 0;
        }

        int index;
        fixed (int* values = counts, selected = active, parts = nodes)
            index = NativeMethods.Reactions(context, Stable(id), icons, values, selected, length,
                enabled ? 1 : 0, focus, parts);
        if (trackingIds is not null)
            for (int i = 0; i < nodes.Length; i++) TrackControl(trackingIds[i], nodes[i]);
        return index < 0 ? new(-1, 0, false) : new(index, counts[index], active[index] != 0);
    }
}
