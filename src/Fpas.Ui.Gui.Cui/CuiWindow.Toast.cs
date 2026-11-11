using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Visible geometry and input state of part of a native toast.</summary>
/// <param name="Bounds">Finalized rectangle in logical points.</param>
/// <param name="Focused">Whether the part has native keyboard focus.</param>
/// <param name="Hovered">Whether the pointer is over the part.</param>
public sealed record CuiToastPart(CuiRectangle Bounds, bool Focused, bool Hovered);

/// <summary>The current native toast and any action delivered in this frame.</summary>
/// <param name="Active">Whether the notification is still pending or visible.</param>
/// <param name="Invoked">Whether the action was activated since the last poll.</param>
/// <param name="Card">Visible card, or null while it is hidden or dismissed.</param>
/// <param name="Action">Visible action button, or null.</param>
/// <param name="Dismiss">Visible close button, or null.</param>
public sealed record CuiToastFrame(bool Active, bool Invoked, CuiToastPart? Card, CuiToastPart? Action, CuiToastPart? Dismiss);

public sealed partial class CuiWindow
{
    /// <summary>Queues one notification for the native CUI overlay.</summary>
    /// <param name="title">Nonblank title, up to 127 UTF-8 bytes.</param>
    /// <param name="description">Optional secondary text, up to 191 UTF-8 bytes.</param>
    /// <param name="tone">Native neutral, info, success, warning or danger tone.</param>
    /// <param name="duration">Seconds until dismissal, zero for the native default, negative to keep.</param>
    /// <param name="action">Optional action caption, up to 47 UTF-8 bytes and without the ## delimiter.</param>
    /// <param name="showTimer">Whether to show the remaining time.</param>
    /// <returns>Handle for polling the toast.</returns>
    public int Toast(string title, string description, int tone, float duration, string action, bool showTimer)
    {
        CheckFrame();
        ArgumentException.ThrowIfNullOrWhiteSpace(title);
        CheckToastText(title, 127, nameof(title));
        CheckToastText(description, 191, nameof(description));
        CheckToastText(action, 47, nameof(action));
        if (action.Length != 0 && (string.IsNullOrWhiteSpace(action) || action.Contains("##", StringComparison.Ordinal)))
            throw new ArgumentException("A toast action needs a visible caption without the ## delimiter.", nameof(action));
        if (!float.IsFinite(duration)) throw new ArgumentOutOfRangeException(nameof(duration));
        if (tone is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(tone));
        return NativeMethods.Toast(context, title, description.Length == 0 ? null : description,
            tone, duration, action.Length == 0 ? null : action, showTimer ? 1 : 0);
    }

    /// <summary>Reads visible toast geometry and consumes the action event after presentation.</summary>
    /// <param name="handle">Native handle returned when queuing the toast.</param>
    /// <returns>The toast's state in the completed frame.</returns>
    public CuiToastFrame PollToast(int handle)
    {
        CheckOpen();
        if (inFrame || !hasPresentedFrame)
            throw new InvalidOperationException("Present the current frame before polling a toast.");
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(handle);
        int flags = NativeMethods.ToastInspect(context, handle, out int card, out int action, out int dismiss);
        return new((flags & 1) != 0, (flags & 2) != 0, ToastPart(card), ToastPart(action), ToastPart(dismiss));
    }

    private CuiToastPart? ToastPart(int node) => node < 0 ? null : new(
        NativeMethods.NodeRectangle(context, node),
        NativeMethods.Focused(context, node) != 0,
        NativeMethods.Hovered(context, node) != 0);

    private static void CheckToastText(string value, int limit, string parameter)
    {
        Utf8.Validate(value, parameter);
        if (Utf8.Encoding.GetByteCount(value) > limit)
            throw new ArgumentException($"Toast text exceeds its {limit}-byte UTF-8 limit.", parameter);
    }
}
