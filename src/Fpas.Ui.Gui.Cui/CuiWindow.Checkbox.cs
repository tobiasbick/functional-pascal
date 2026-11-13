using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's checkbox with a literal caption and stable identity.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="description">Optional secondary text.</param>
    /// <param name="checked">Current model-owned value.</param>
    /// <param name="enabled">Whether the row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="indeterminate">Draw a dash in place of the check mark while checked.</param>
    /// <param name="hideLabel">Draws the checkbox without the visible caption.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public bool Checkbox(string id, string label, string description, bool @checked, bool enabled, bool focus, bool indeterminate, bool hideLabel = false)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(description, nameof(description));
        ArgumentException.ThrowIfNullOrEmpty(label);
        return NativeMethods.Checkbox(context, Stable(id), label, description.Length == 0 ? null : description,
            @checked ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, indeterminate ? 1 : 0, hideLabel ? 1 : 0) != 0;
    }
}
