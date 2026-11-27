// [Documentation](../../../../docs/ui/gui/selection/chip.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI chip with a literal caption and optional model-owned selection.</summary>
    /// <param name="id">Stable chip identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="icon">Glyph for the unselected state, or null to omit it.</param>
    /// <param name="selected">Current selection, or null for a stateless chip.</param>
    /// <param name="enabled">Whether the chip accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether CUI activated the chip.</returns>
    public bool Chip(string id, string label, string? icon, bool? selected, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        ArgumentException.ThrowIfNullOrWhiteSpace(label);
        int selection = selected.HasValue ? (selected.Value ? 1 : 0) : -1;
        return NativeMethods.Chip(context, Stable(id), label, icon,
            selection, enabled ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
