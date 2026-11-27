// [Documentation](../../../../docs/ui/gui/actions/icon-button.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native icon button with a stable identity.</summary>
    /// <param name="id">Identity independent of the icon and tooltip.</param>
    /// <param name="icon">Glyph from the native icon font.</param>
    /// <param name="tooltip">Accessible name and pointer tooltip.</param>
    /// <param name="shortcut">Optional shortcut hint.</param>
    /// <param name="variant">Shared semantic button variant ordinal.</param>
    /// <param name="size">Medium, small or large in native enum order.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="loading">Whether activation is suspended while busy.</param>
    /// <param name="selected">Whether to paint the button as selected.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="widthKind">Content, fill, fixed or percentage in that order.</param>
    /// <param name="width">Fixed logical width or percentage, when selected.</param>
    /// <param name="rgba">Packed RGBA foreground override, or zero for the default.</param>
    /// <returns>Whether the button activated.</returns>
    public bool IconButton(string id, string icon, string tooltip, string shortcut, int variant, int size,
        bool enabled, bool loading, bool selected, bool focus, int widthKind = 0, float width = 0, uint rgba = 0)
    {
        CheckFrame();
        Utf8.Validate(icon, nameof(icon));
        CheckTooltipText(tooltip, shortcut, nameof(tooltip));
        ArgumentException.ThrowIfNullOrWhiteSpace(icon);
        if (variant is < 0 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        CheckExtent(widthKind, width, nameof(width));
        return NativeMethods.IconButton(context, Stable(id), icon, tooltip, Optional(shortcut),
            variant, size, enabled ? 1 : 0, loading ? 1 : 0, selected ? 1 : 0, focus ? 1 : 0,
            widthKind, width, rgba) != 0;
    }
}
