// [Documentation](../../../../docs/ui/gui/selection/switch.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's switch with a literal caption and stable identity.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="description">Optional secondary text.</param>
    /// <param name="on">Current model-owned value.</param>
    /// <param name="enabled">Whether the row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="hideLabel">Whether to draw the switch track without its caption.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public bool Switch(string id, string label, string? description, bool on, bool enabled, bool focus, bool hideLabel = false)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        if (description is not null) Utf8.Validate(description, nameof(description));
        ArgumentException.ThrowIfNullOrEmpty(label);
        return NativeMethods.Switch(context, Stable(id), label, description,
            on ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, hideLabel ? 1 : 0) != 0;
    }
}
