using Fpas.Ui.Gui.Cui.Widgets.Feedback;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
