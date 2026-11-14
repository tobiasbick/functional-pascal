using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's analog hour and minute dial.</summary>
    /// <param name="id">Stable dial identity.</param>
    /// <param name="value">Model time at minute precision.</param>
    /// <param name="size">Face diameter in logical points, or zero for CUI's default.</param>
    /// <param name="enabled">Whether the dial accepts input.</param>
    /// <param name="focus">Zero for no request, one for hours or two for minutes.</param>
    /// <param name="hours">Hours selector tracking ID, or null to skip tracking.</param>
    /// <param name="minutes">Minutes selector tracking ID.</param>
    /// <param name="face">Clock face tracking ID.</param>
    /// <returns>The proposed time.</returns>
    public unsafe TimeOnly TimeDial(string id, TimeOnly value, float size, bool enabled, int focus,
        string? hours, string minutes, string face)
    {
        CheckFrame();
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(focus));
        int* nodes = stackalloc int[4];
        int proposed = NativeMethods.TimeDial(context, Stable(id), MinuteOfDay(value), size,
            enabled ? 1 : 0, focus, hours is null ? null : nodes);
        if (hours is not null)
        {
            TrackControl(id, nodes[0]);
            TrackControl(hours, nodes[1]);
            TrackControl(minutes, nodes[2]);
            TrackControl(face, nodes[3]);
        }
        return new TimeOnly(proposed / 60, proposed % 60);
    }
}
