using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native outline for an uncontrolled host viewport.</summary>
    /// <param name="id">Stable outline identity for inspection.</param>
    /// <param name="scrollId">Identity passed to <see cref="BeginViewport(string, bool, bool)"/> in the same CUI scope.</param>
    /// <param name="titles">One to 64 entries in document order.</param>
    /// <param name="anchors">The matching globally unique anchor identities.</param>
    /// <param name="levels">Indentation levels from zero to four.</param>
    /// <param name="enabled">Whether entries accept input.</param>
    /// <param name="focus">One plus the row to focus, or zero for no request.</param>
    /// <param name="trackRow">Maps a row index to its inspection identity, or null to skip tracking.</param>
    /// <returns>The index of the active entry.</returns>
    public unsafe int TableOfContents(string id, string scrollId, string[] titles, string[] anchors,
        int[] levels, bool enabled, int focus, Func<int, string>? trackRow)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(titles);
        ArgumentNullException.ThrowIfNull(anchors);
        ArgumentNullException.ThrowIfNull(levels);
        int count = titles.Length;
        if (count is < 1 or > 64 || anchors.Length != count || levels.Length != count)
            throw new ArgumentException("The outline needs one to 64 matching titles, anchors and levels.");
        if (focus < 0 || focus > count) throw new ArgumentOutOfRangeException(nameof(focus));
        for (int index = 0; index < count; ++index)
        {
            Utf8.Validate(titles[index], nameof(titles));
            Utf8.Validate(anchors[index], nameof(anchors));
            if (string.IsNullOrWhiteSpace(titles[index])) throw new ArgumentException("An outline row needs a title.", nameof(titles));
            ArgumentException.ThrowIfNullOrEmpty(anchors[index], nameof(anchors));
            if (levels[index] is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(levels));
        }

        Span<int> rows = trackRow is null ? [] : stackalloc int[count];
        int active, node;
        fixed (int* depths = levels, parts = rows)
            active = NativeMethods.TableOfContents(context, Stable(scrollId), titles, anchors, depths,
                count, enabled ? 1 : 0, focus, out node, parts);

        if (trackRow is null) return active;
        TrackControl(id, node);
        for (int index = 0; index < count; ++index) TrackControl(trackRow(index), rows[index]);
        return active;
    }
}
