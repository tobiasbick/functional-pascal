using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Reports activation and the native confirmation state after drawing.</summary>
/// <param name="Confirmed">Whether this activation completed the confirmation.</param>
/// <param name="Armed">Whether the control is waiting for a second activation.</param>
public readonly record struct CuiConfirmButtonResult(bool Confirmed, bool Armed);

public sealed partial class CuiWindow
{
    private readonly HashSet<uint> confirmButtons = [];

    /// <summary>Draws CUI's two-step button with literal captions and a stable identity.</summary>
    /// <param name="id">Identity independent of both captions.</param>
    /// <param name="label">Literal resting caption.</param>
    /// <param name="confirmLabel">Literal confirmation caption.</param>
    /// <param name="icon">Optional resting glyph.</param>
    /// <param name="timeoutMilliseconds">Positive confirmation window.</param>
    /// <param name="variant">Shared resting button variant ordinal.</param>
    /// <param name="size">Medium, small or large size ordinal.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Confirmation and armed state for this frame.</returns>
    public CuiConfirmButtonResult ConfirmButton(string id, string label, string confirmLabel, string icon,
        int timeoutMilliseconds, int variant, int size, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(confirmLabel, nameof(confirmLabel));
        Utf8.Validate(icon, nameof(icon));
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(timeoutMilliseconds);
        if (variant is < 0 or 3 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        int events = NativeMethods.ConfirmButton(context, Stable(id), label, confirmLabel,
            icon.Length == 0 ? null : icon, timeoutMilliseconds, variant, size, enabled ? 1 : 0, focus ? 1 : 0);
        confirmButtons.Add(NativeMethods.NodeKey(context, NativeMethods.LastNode(context)));
        return new((events & 1) != 0, (events & 2) != 0);
    }

    private void ForgetConfirmButton(uint key)
    {
        if (confirmButtons.Remove(key)) NativeMethods.Forget(context, key ^ 0xC0F1u);
    }
}
