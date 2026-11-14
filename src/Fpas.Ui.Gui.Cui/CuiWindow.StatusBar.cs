using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native status item result.</summary>
/// <param name="Shown">Whether the item fits and was drawn.</param>
/// <param name="Clicked">Whether the item was activated.</param>
public readonly record struct CuiStatusBarResult(bool Shown, bool Clicked);

public sealed partial class CuiWindow
{
    /// <summary>Opens a native status bar.</summary>
    /// <param name="id">Stable bar identity.</param>
    /// <param name="border">Whether to draw a top hairline.</param>
    public void BeginStatusBar(string id, bool border)
    {
        CheckFrame();
        NativeMethods.StatusBarBegin(context, Stable(id), border ? 1 : 0);
    }

    /// <summary>Adds a static message to the current status bar.</summary>
    /// <param name="text">Visible text.</param>
    /// <param name="icon">Optional icon glyph.</param>
    /// <param name="tone">Native icon color.</param>
    public void StatusBarText(string text, string icon, int tone)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(icon, nameof(icon));
        NativeMethods.StatusBarText(context, text, Optional(icon), tone);
    }

    /// <summary>Adds a clickable item that fits in the current bar.</summary>
    /// <param name="id">Stable item key.</param>
    /// <param name="label">Visible caption.</param>
    /// <param name="icon">Optional icon glyph.</param>
    /// <param name="tone">Native icon color.</param>
    /// <param name="tooltip">Optional tooltip.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">Whether to request focus.</param>
    /// <param name="trackingId">Inspection identity, or null.</param>
    /// <returns>Whether CUI drew and activated the item.</returns>
    public CuiStatusBarResult StatusBarItem(string id, string label, string icon, int tone,
        string tooltip, bool enabled, bool focus, string? trackingId)
    {
        CheckFrame();
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(tooltip, nameof(tooltip));
        int click = NativeMethods.StatusBarItem(context, ChromeTitle(label, id), Optional(icon),
            tone, Optional(tooltip), enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (node >= 0 && trackingId is not null) TrackControl(trackingId, node);
        return new(node >= 0, click != 0);
    }

    /// <summary>Adds a progress indicator to the current bar.</summary>
    /// <param name="value">Progress from zero to one.</param>
    /// <param name="label">Optional caption.</param>
    /// <param name="width">Width in logical points, or zero for the native default.</param>
    /// <param name="indeterminate">Whether to show the indeterminate animation.</param>
    public void StatusBarProgress(float value, string label, float width, bool indeterminate)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        CheckFeedbackNumber(width, nameof(width));
        if (!float.IsFinite(value) || value is < 0 or > 1)
            throw new ArgumentOutOfRangeException(nameof(value));
        NativeMethods.StatusBarProgress(context, value, Optional(label), width, indeterminate ? 1 : 0);
    }

    /// <summary>Sends subsequent entries to the far side of the current bar.</summary>
    public void StatusBarSpacer()
    {
        CheckFrame();
        NativeMethods.StatusBarSpacer(context);
    }

    /// <summary>Closes the native status bar.</summary>
    public void EndStatusBar()
    {
        CheckFrame();
        NativeMethods.StatusBarEnd(context);
    }
}
