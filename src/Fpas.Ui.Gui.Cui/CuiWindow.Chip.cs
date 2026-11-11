using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI chip with a literal caption and model-owned selection.</summary>
    /// <param name="id">Stable chip identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="icon">Glyph for the unselected state, or an empty string.</param>
    /// <param name="selected">Current selection.</param>
    /// <param name="enabled">Whether the chip accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether CUI proposed toggling the chip.</returns>
    public bool Chip(string id, string label, string icon, bool selected, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        ArgumentException.ThrowIfNullOrWhiteSpace(label);
        return NativeMethods.Chip(context, Stable(id), label, icon.Length == 0 ? null : icon,
            selected ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
