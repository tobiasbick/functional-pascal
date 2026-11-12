using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's rolling countdown digits for the supplied duration.</summary>
    /// <param name="id">Stable countdown identity.</param>
    /// <param name="secondsLeft">Finite seconds remaining. Negative values display zero.</param>
    /// <param name="compact">Whether to draw the compact tiles.</param>
    /// <param name="hideDays">Whether to fold days into hours.</param>
    /// <param name="track">Whether to retain the countdown bounds.</param>
    public void Countdown(string id, double secondsLeft, bool compact, bool hideDays, bool track)
    {
        CheckFrame();
        if (!double.IsFinite(secondsLeft) || secondsLeft > int.MaxValue * 3600.0 + 3599)
            throw new ArgumentOutOfRangeException(nameof(secondsLeft));
        int node = NativeMethods.Countdown(context, Stable(id), secondsLeft, compact ? 1 : 0, hideDays ? 1 : 0);
        if (track) TrackControl(id, node);
    }
}
