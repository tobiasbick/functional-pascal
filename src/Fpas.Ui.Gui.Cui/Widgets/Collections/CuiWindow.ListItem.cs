// [Documentation](../../../../docs/ui/gui/data-display/list-item.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native list item and returns activation.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="title">Literal primary text.</param>
    /// <param name="subtitle">Optional secondary text.</param>
    /// <param name="trailing">Optional trailing label.</param>
    /// <param name="avatar">Optional name for the initials avatar.</param>
    /// <param name="icon">Optional leading glyph.</param>
    /// <param name="selected">Whether to show selection.</param>
    /// <param name="enabled">Whether the row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether the row was activated.</returns>
    public bool ListItem(string id, string title, string? subtitle, string? trailing, string? avatar,
        string? icon, bool selected, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        if (subtitle is not null) Utf8.Validate(subtitle, nameof(subtitle));
        if (trailing is not null) Utf8.Validate(trailing, nameof(trailing));
        if (avatar is not null) Utf8.Validate(avatar, nameof(avatar));
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        return NativeMethods.ListItem(context, Stable(id), title, subtitle, trailing, avatar, icon,
            selected ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
