using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native CUI segments and returns a model proposal.</summary>
    /// <param name="id">Stable group identity.</param>
    /// <param name="labels">One through 32 literal labels, with empty text for icon-only segments.</param>
    /// <param name="icons">One glyph per segment, empty for no glyph, or null for text-only segments.</param>
    /// <param name="selected">Model-owned selected ordinal.</param>
    /// <param name="size">Medium, small or large CUI control size.</param>
    /// <param name="fullWidth">Whether the segments fill their allocation.</param>
    /// <param name="enabled">Whether input and focus are permitted.</param>
    /// <param name="focus">Zero for none, or one plus the requested focus ordinal.</param>
    /// <param name="trackingIds">Group and ordered segment tracking IDs, or null.</param>
    /// <returns>Minus one without a change; otherwise the proposed ordinal.</returns>
    public unsafe int Segmented(string id, string[] labels, string[]? icons, int selected, int size,
        bool fullWidth, bool enabled, int focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        int count = labels.Length;
        if (count is < 1 or > 32) throw new ArgumentException("Supply one through 32 segments.", nameof(labels));
        if (icons is not null && icons.Length != count) throw new ArgumentException("Supply one icon per segment.", nameof(icons));
        if (selected < 0 || selected >= count) throw new ArgumentOutOfRangeException(nameof(selected));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus < 0 || focus > count) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != count + 1)
            throw new ArgumentException("Supply group and segment tracking IDs.", nameof(trackingIds));
        foreach (string label in labels) Utf8.Validate(label, nameof(labels));
        if (icons is not null) foreach (string icon in icons) Utf8.Validate(icon, nameof(icons));
        int[] nodes = new int[count + 1];
        int proposal;
        fixed (int* parts = nodes)
            proposal = NativeMethods.Segmented(context, Stable(id), labels, icons, count, selected, size,
                fullWidth ? 1 : 0, enabled ? 1 : 0, focus, parts);
        if (trackingIds is not null)
            for (int index = 0; index < nodes.Length; index++) TrackControl(trackingIds[index], nodes[index]);
        return proposal;
    }
}
