using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's radio button with a literal caption and stable identity.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="description">Optional secondary text.</param>
    /// <param name="selected">Current model-owned value.</param>
    /// <param name="enabled">Whether the row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public bool RadioButton(string id, string label, string description, bool selected, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(description, nameof(description));
        ArgumentException.ThrowIfNullOrEmpty(label);
        return NativeMethods.RadioButton(context, Stable(id), label, description.Length == 0 ? null : description,
            selected ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
