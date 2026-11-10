using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native hold-to-confirm button with a stable identity.</summary>
    /// <param name="id">Identity independent of the label.</param>
    /// <param name="label">Literal button caption.</param>
    /// <param name="icon">Optional native glyph.</param>
    /// <param name="durationMilliseconds">Time the action must be held.</param>
    /// <param name="variant">Shared semantic button variant ordinal.</param>
    /// <param name="enabled">Whether the button accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>True once when the hold completes.</returns>
    public bool HoldButton(string id, string label, string icon, int durationMilliseconds, int variant, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(durationMilliseconds);
        if (variant is < 0 or 3 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        return NativeMethods.HoldButton(context, Stable(id), label, icon.Length == 0 ? null : icon,
            durationMilliseconds, variant, enabled ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
