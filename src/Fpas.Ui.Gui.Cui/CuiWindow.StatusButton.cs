using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private readonly HashSet<uint> statusButtons = [];

    /// <summary>Draws native inline work feedback with literal captions and stable focus.</summary>
    /// <param name="id">Identity independent of state and captions.</param>
    /// <param name="label">Literal idle caption.</param>
    /// <param name="status">Idle, busy, success or error ordinal.</param>
    /// <param name="busyLabel">Literal busy caption.</param>
    /// <param name="successLabel">Literal success caption.</param>
    /// <param name="errorLabel">Literal error caption.</param>
    /// <param name="icon">Optional idle glyph.</param>
    /// <param name="variant">Shared resting button variant ordinal.</param>
    /// <param name="size">Medium, small or large size ordinal.</param>
    /// <param name="enabled">Whether the control accepts focus and idle input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>True only for activation while idle.</returns>
    public bool StatusButton(string id, string label, int status, string busyLabel, string successLabel,
        string errorLabel, string icon, int variant, int size, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(busyLabel, nameof(busyLabel));
        Utf8.Validate(successLabel, nameof(successLabel));
        Utf8.Validate(errorLabel, nameof(errorLabel));
        Utf8.Validate(icon, nameof(icon));
        if (status is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(status));
        if (variant is < 0 or 3 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        int activated = NativeMethods.StatusButton(context, Stable(id), label, status, busyLabel,
            successLabel, errorLabel, icon.Length == 0 ? null : icon, variant, size, enabled ? 1 : 0, focus ? 1 : 0);
        statusButtons.Add(NativeMethods.NodeKey(context, NativeMethods.LastNode(context)));
        return activated != 0;
    }

    private void ForgetStatusButton(uint key)
    {
        if (statusButtons.Remove(key)) NativeMethods.Forget(context, key ^ 0x57A7u);
    }
}
