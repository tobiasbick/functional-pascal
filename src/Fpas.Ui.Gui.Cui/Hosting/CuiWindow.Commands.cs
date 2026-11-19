// [Documentation](../../../docs/ui/gui/cui.md#window-ownership-and-frames)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Changes the native title on the window's owner thread.</summary>
    /// <param name="title">Valid Unicode title, copied by CUI.</param>
    public void SetTitle(string title)
    {
        CheckOpen();
        Utf8.Validate(title, nameof(title));
        NativeMethods.SetTitle(app, title);
    }

    /// <summary>Requests one redraw now, or a wakeup after a delay; native earlier deadlines take priority.</summary>
    /// <param name="delaySeconds">Finite nonnegative delay in seconds; zero requests the next frame.</param>
    public void RequestFrame(float delaySeconds = 0)
    {
        CheckOpen();
        if (!float.IsFinite(delaySeconds) || delaySeconds < 0)
            throw new ArgumentOutOfRangeException(nameof(delaySeconds));
        if (delaySeconds == 0) NativeMethods.RequestRedraw(context);
        else NativeMethods.RequestWakeup(context, delaySeconds);
    }

    /// <summary>Consumes one deferred native close request without stopping the window.</summary>
    /// <returns>Whether a close request was pending.</returns>
    public bool TakeCloseRequest()
    {
        CheckOpen();
        bool requested = CloseRequested;
        CloseRequested = false;
        return requested;
    }
}
