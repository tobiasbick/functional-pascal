using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Actions and presence of the controls within a native banner.</summary>
/// <param name="Activated">Whether the action was clicked.</param>
/// <param name="Dismissed">Whether the close button was clicked.</param>
/// <param name="Visible">Whether the banner still occupies space.</param>
/// <param name="HasAction">Whether the action button was drawn.</param>
/// <param name="HasDismiss">Whether the dismiss button was drawn.</param>
public readonly record struct CuiBannerResult(bool Activated, bool Dismissed, bool Visible, bool HasAction, bool HasDismiss);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native announcement banner and its optional controls.</summary>
    /// <param name="id">Stable banner identity.</param>
    /// <param name="text">Literal announcement.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <param name="action">Optional action caption.</param>
    /// <param name="icon">Optional icon glyph.</param>
    /// <param name="tone">Native semantic tone.</param>
    /// <param name="dismissible">Whether to show the close button.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">One for action, two for dismiss, zero for neither.</param>
    /// <param name="actionId">Optional inspection identity of the action.</param>
    /// <param name="dismissId">Optional inspection identity of the dismiss button.</param>
    /// <param name="track">Whether to retain the banner and its bounds.</param>
    /// <returns>Native proposals and visibility.</returns>
    public CuiBannerResult Banner(string id, string text, bool open, string action, string icon, int tone,
        bool dismissible, bool enabled, int focus, string actionId, string dismissId, bool track)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(action, nameof(action));
        Utf8.Validate(icon, nameof(icon));
        CheckTone(tone);
        if (string.IsNullOrWhiteSpace(text)) throw new ArgumentException("A banner needs an announcement.", nameof(text));
        if (focus is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(focus));
        int result = NativeMethods.Banner(context, Stable(id), text, open ? 1 : 0, Optional(action), Optional(icon),
            tone, dismissible ? 1 : 0, enabled ? 1 : 0, focus, out int node, out int actionNode,
            out int dismissNode, out uint key);
        if (track)
        {
            TrackControl(id, node, key);
            if (actionNode >= 0) TrackControl(actionId, actionNode);
            if (dismissNode >= 0) TrackControl(dismissId, dismissNode);
        }
        return new((result & 1) != 0, (result & 2) != 0, node >= 0, actionNode >= 0, dismissNode >= 0);
    }
}
