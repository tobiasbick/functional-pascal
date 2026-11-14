using Fpas.Ui.Gui.Cui.Widgets.Actions;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native split button and optionally registers its visible parts.</summary>
    /// <param name="id">Stable native identity.</param>
    /// <param name="label">Literal main action caption.</param>
    /// <param name="icon">Optional main glyph.</param>
    /// <param name="labels">Ordered literal menu captions.</param>
    /// <param name="icons">Ordered menu glyphs, with empty strings for absent icons.</param>
    /// <param name="trackingIds">Main, chevron and item tracking IDs, or null to skip tracking.</param>
    /// <param name="variant">Shared CUI button variant ordinal.</param>
    /// <param name="size">CUI control size.</param>
    /// <param name="enabled">Whether either half accepts input.</param>
    /// <param name="focus">Zero for no request, one for main, or two for the chevron.</param>
    /// <returns>The selected action, visibility and highlight.</returns>
    public unsafe CuiSplitButtonResult SplitButton(string id, string label, string icon, string[] labels,
        string[] icons, string[]? trackingIds, int variant, int size, bool enabled, int focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        ArgumentNullException.ThrowIfNull(labels);
        ArgumentNullException.ThrowIfNull(icons);
        if (labels.Length == 0 || labels.Length != icons.Length)
            throw new ArgumentException("Each menu item needs a label and an icon slot.", nameof(labels));
        if (trackingIds is not null && trackingIds.Length != labels.Length + 2)
            throw new ArgumentException("Supply tracking IDs for both halves and every menu item.", nameof(trackingIds));
        if (variant is < 0 or 3 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(focus));
        for (int index = 0; index < labels.Length; index++)
        {
            Utf8.Validate(labels[index], nameof(labels));
            Utf8.Validate(icons[index], nameof(icons));
        }

        int[] nodes = new int[labels.Length + 2];
        int choice, open, highlighted;
        fixed (int* parts = nodes)
            choice = NativeMethods.SplitButton(context, Stable(id), label, icon.Length == 0 ? null : icon,
                labels, icons, labels.Length, variant, size, enabled ? 1 : 0, focus, parts, out open, out highlighted);
        if (trackingIds is not null)
        {
            for (int index = 0; index < nodes.Length; index++)
                if (nodes[index] >= 0) TrackControl(trackingIds[index], nodes[index]);
        }

        return new CuiSplitButtonResult(choice, open != 0, highlighted);
    }
}
