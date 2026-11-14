using System.Numerics;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI toggle group and returns a changed selection.</summary>
    /// <param name="id">Stable group identity.</param>
    /// <param name="labels">One through 32 literal item labels.</param>
    /// <param name="icons">One glyph per item, empty for no glyph, or null for text-only items.</param>
    /// <param name="selected">One model-owned flag per item.</param>
    /// <param name="size">Medium, small or large CUI control size.</param>
    /// <param name="exclusive">Whether exactly one item is selected.</param>
    /// <param name="iconOnly">Whether items with icons show their labels as tooltips.</param>
    /// <param name="enabled">Whether the items receive input and focus.</param>
    /// <param name="focus">Zero for none, or one plus the requested focus ordinal.</param>
    /// <param name="trackingIds">Group and ordered item tracking IDs, or null.</param>
    /// <returns>A new array of flags on a change, or null.</returns>
    public unsafe bool[]? ToggleGroup(string id, string[] labels, string[]? icons, bool[] selected, int size,
        bool exclusive, bool iconOnly, bool enabled, int focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(selected);
        int count = labels.Length;
        if (count is < 1 or > 32) throw new ArgumentException("Supply one through 32 toggles.", nameof(labels));
        if (icons is not null && icons.Length != count) throw new ArgumentException("Supply one icon per toggle.", nameof(icons));
        if (selected.Length != count) throw new ArgumentException("Supply one flag per toggle.", nameof(selected));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus < 0 || focus > count) throw new ArgumentOutOfRangeException(nameof(focus));
        if (trackingIds is not null && trackingIds.Length != count + 1)
            throw new ArgumentException("Supply group and item tracking IDs.", nameof(trackingIds));
        uint mask = 0;
        for (int index = 0; index < count; index++)
        {
            string label = labels[index];
            Utf8.Validate(label, nameof(labels));
            if (label.Length == 0 || label.Contains('\n') || label.Contains('\r'))
                throw new ArgumentException("Supply single-line accessible labels.", nameof(labels));
            if (selected[index]) mask |= 1u << index;
        }
        if (icons is not null) foreach (string icon in icons) Utf8.Validate(icon, nameof(icons));
        if (exclusive && !BitOperations.IsPow2(mask))
            throw new ArgumentException("An exclusive group needs exactly one selected toggle.", nameof(selected));

        Span<int> nodes = stackalloc int[count + 1];
        uint proposal;
        fixed (int* parts = nodes)
            proposal = NativeMethods.ToggleGroup(context, Stable(id), labels, icons, count, mask, size,
                exclusive ? 1 : 0, iconOnly ? 1 : 0, enabled ? 1 : 0, focus, parts);
        if (trackingIds is not null)
            for (int index = 0; index < nodes.Length; index++) TrackControl(trackingIds[index], nodes[index]);
        if (proposal == mask) return null;

        var result = new bool[count];
        for (int index = 0; index < count; index++) result[index] = (proposal & (1u << index)) != 0;
        return result;
    }
}
