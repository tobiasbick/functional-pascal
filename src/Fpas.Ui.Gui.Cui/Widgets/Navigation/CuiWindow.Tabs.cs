using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native tab strip with a model-owned selection.</summary>
    /// <param name="id">Stable strip identity.</param>
    /// <param name="labels">One through 32 literal captions.</param>
    /// <param name="icons">One glyph per tab, with empty strings for text-only tabs.</param>
    /// <param name="selected">Current selected index.</param>
    /// <param name="variant">Zero for underline or one for pill.</param>
    /// <param name="fullWidth">Whether the tabs fill the available width.</param>
    /// <param name="enabled">Whether the strip accepts input.</param>
    /// <param name="focus">Zero or one plus the requested tab index.</param>
    /// <param name="trackingIds">Strip and tab tracking IDs, or null.</param>
    /// <returns>The proposed index, or minus one when unchanged.</returns>
    public unsafe int Tabs(string id, string[] labels, string[] icons, int selected, int variant,
        bool fullWidth, bool enabled, int focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        int count = labels.Length;
        if (count is < 1 or > 32) throw new ArgumentException("Supply one through 32 tabs.", nameof(labels));
        if (icons.Length != count) throw new ArgumentException("Supply one glyph per tab.", nameof(icons));
        if (selected < 0 || selected >= count) throw new ArgumentOutOfRangeException(nameof(selected));
        if (variant is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(variant));
        if (focus < 0 || focus > count) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != count + 1)
            throw new ArgumentException("Supply strip and tab tracking IDs.", nameof(trackingIds));
        foreach (string label in labels)
        {
            Utf8.Validate(label, nameof(labels));
            ArgumentException.ThrowIfNullOrWhiteSpace(label, nameof(labels));
        }
        foreach (string icon in icons) Utf8.Validate(icon, nameof(icons));
        int* nodes = stackalloc int[count + 1];
        int proposal = NativeMethods.Tabs(context, Stable(id), labels, icons, count, selected, variant,
            fullWidth ? 1 : 0, enabled ? 1 : 0, focus, nodes);
        if (trackingIds is not null)
            for (int index = 0; index <= count; index++) TrackControl(trackingIds[index], nodes[index]);
        return proposal;
    }
}
