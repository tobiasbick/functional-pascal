// [Documentation](../../../../docs/ui/gui/feedback/empty-state.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native empty state with its optional action.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="title">Literal heading.</param>
    /// <param name="icon">CUI glyph. Null omits its area and an empty string reserves a blank area.</param>
    /// <param name="description">Supporting text. Null omits it and an empty string reserves a blank line.</param>
    /// <param name="action">Action caption. Null omits the button and an empty string shows its plus icon.</param>
    /// <param name="enabled">Whether the action accepts input.</param>
    /// <param name="focus">Whether to focus the action.</param>
    /// <param name="trackAction">Optional inspection identity for the action.</param>
    /// <returns>Whether the action was activated.</returns>
    public bool EmptyState(string id, string title, string? icon, string? description, string? action,
        bool enabled, bool focus, string? trackAction)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        if (description is not null) Utf8.Validate(description, nameof(description));
        if (action is not null) Utf8.Validate(action, nameof(action));
        int clicked = NativeMethods.EmptyState(context, Stable(id), title, icon,
            description, action, enabled ? 1 : 0, focus ? 1 : 0, out int actionNode);
        if (trackAction is not null && actionNode >= 0) TrackControl(trackAction, actionNode);
        return clicked != 0;
    }
}
