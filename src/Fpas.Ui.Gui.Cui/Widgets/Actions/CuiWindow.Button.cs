using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Adds a button and reports activation.</summary>
    /// <param name="label">CUI label including its stable identity suffix.</param>
    /// <param name="enabled">Whether the control accepts pointer and keyboard input.</param>
    /// <returns>Whether the button was activated.</returns>
    public bool Button(string label, bool enabled = true)
    {
        CheckFrame(); Utf8.Validate(label, nameof(label));
        NativeMethods.PushDisabled(context, enabled ? (byte)0 : (byte)1);
        try { return NativeMethods.Button(context, label, 0) != 0; }
        finally { NativeMethods.PopDisabled(context); }
    }

    /// <summary>Draws a native button with a literal label and a stable identity.</summary>
    /// <param name="id">Identity independent of the label.</param>
    /// <param name="label">Literal visible label.</param>
    /// <param name="variant">Shared semantic button variant ordinal.</param>
    /// <param name="size">Medium, small or large in native enum order.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="loading">Whether activation is suspended while busy.</param>
    /// <param name="selected">Whether to paint the button as selected.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="icon">Leading native glyph, or an empty string.</param>
    /// <param name="iconRight">Trailing native glyph, or an empty string.</param>
    /// <param name="tooltip">Hover tooltip, or an empty string.</param>
    /// <param name="shortcut">Shortcut hint, or an empty string.</param>
    /// <param name="fill">Whether to fill the available width.</param>
    /// <param name="width">Fixed logical width, or -1 for intrinsic width.</param>
    /// <param name="rgba">Packed RGBA foreground override, or zero for the default.</param>
    /// <returns>Whether the button activated.</returns>
    public bool Button(string id, string label, int variant, int size, bool enabled, bool loading,
        bool selected, bool focus, string icon, string iconRight, string tooltip, string shortcut,
        bool fill, float width, uint rgba)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(iconRight, nameof(iconRight));
        Utf8.Validate(tooltip, nameof(tooltip));
        Utf8.Validate(shortcut, nameof(shortcut));
        if (variant is < 0 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        if (!float.IsFinite(width) || (width < 0 && width != -1)) throw new ArgumentOutOfRangeException(nameof(width));
        return NativeMethods.HostButton(context, Stable(id), label, variant, size, enabled ? 1 : 0, loading ? 1 : 0,
            selected ? 1 : 0, focus ? 1 : 0, icon.Length == 0 ? null : icon, iconRight.Length == 0 ? null : iconRight,
            tooltip.Length == 0 ? null : tooltip, shortcut.Length == 0 ? null : shortcut, fill ? 1 : 0, width, rgba) != 0;
    }
}
