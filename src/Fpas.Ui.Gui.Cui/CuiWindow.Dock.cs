using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native dock with item snapshots.</summary>
    /// <param name="id">Stable dock identity.</param>
    /// <param name="icons">Glyph per item. Separators use an empty string.</param>
    /// <param name="labels">Labels in entry order.</param>
    /// <param name="colors">Colors in entry order.</param>
    /// <param name="badges">Badge counts in entry order.</param>
    /// <param name="kinds">Separator zero, item one, running item two.</param>
    /// <param name="size">Base icon size, or zero.</param>
    /// <param name="magnify">Magnification, or zero.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">Entry to focus, or minus one.</param>
    /// <param name="part">Optional snapshot identity by entry index.</param>
    /// <returns>The activated entry index, or minus one.</returns>
    public unsafe int Dock(string id, string[] icons, string[] labels, CuiChartColor[] colors, int[] badges,
        int[] kinds, float size, float magnify, bool enabled, int focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckFeedbackNumber(size, nameof(size));
        CheckFeedbackNumber(magnify, nameof(magnify));
        int count = icons.Length;
        if (labels.Length != count || colors.Length != count || badges.Length != count || kinds.Length != count)
            throw new ArgumentException("Expected matching dock entries.");
        var palettes = new int[count];
        var rgba = new uint[count];
        var nodes = new int[count];
        for (int i = 0; i < count; ++i)
        {
            Utf8.Validate(icons[i], nameof(icons));
            Utf8.Validate(labels[i], nameof(labels));
            palettes[i] = colors[i].Palette;
            rgba[i] = colors[i].Rgba;
        }
        int chosen;
        fixed (int* p = palettes, b = badges, k = kinds, n = nodes)
        fixed (uint* c = rgba)
            chosen = NativeMethods.Dock(context, Stable(id), icons, labels, p, c, b, k,
                count, size, magnify, enabled ? 1 : 0, focus, n);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return chosen;
    }
}
