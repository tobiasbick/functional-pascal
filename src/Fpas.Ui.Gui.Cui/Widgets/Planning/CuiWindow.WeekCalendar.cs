using Fpas.Ui.Gui.Cui.Interop.Planning;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Planning;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native week calendar.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="week">First day number.</param>
    /// <param name="events">Current appointments.</param>
    /// <param name="start">First visible hour.</param>
    /// <param name="finish">Last visible hour.</param>
    /// <param name="hourHeight">Height per hour.</param>
    /// <param name="heightKind">Native height kind.</param>
    /// <param name="height">Preferred height.</param>
    /// <param name="flags">Native presentation and input flags.</param>
    /// <param name="focus">Requested native part.</param>
    /// <param name="part">Tracks visible native parts.</param>
    /// <returns>Native model proposal.</returns>
    public unsafe CuiWeekResult WeekCalendar(string id, int week, CuiWeekEvent[] events,
        float start, float finish, float hourHeight, int heightKind, float height, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckExtent(heightKind, height, nameof(height));
        using var memory = new ChartMemory();
        var data = new NativeWeekEvent[events.Length];
        for (int i = 0; i < data.Length; ++i)
        {
            var e = events[i];
            Utf8.Validate(e.Title, nameof(events));
            Utf8.Validate(e.Detail, nameof(events));
            data[i] = new() { Title = memory.Text(e.Title), Detail = memory.Text(Optional(e.Detail)), Day = e.Day,
                Start = e.Start, Finish = e.Finish, Tone = e.Tone };
        }
        var nodes = new int[data.Length];
        int activated;
        fixed (NativeWeekEvent* values = data)
        fixed (int* parts = nodes)
            activated = NativeMethods.WeekCalendar(context, Stable(id), week, values, data.Length, start, finish,
                hourHeight, heightKind, height, flags, focus, parts);
        CuiWeekEvent[]? edited = null;
        for (int i = 0; i < data.Length; ++i)
        {
            var e = events[i];
            var value = data[i];
            if (e.Day != value.Day || e.Start != value.Start || e.Finish != value.Finish)
            {
                edited ??= (CuiWeekEvent[])events.Clone();
                edited[i] = e with { Day = value.Day, Start = value.Start, Finish = value.Finish };
            }
        }
        return new(edited, activated, TrackScheduleParts(nodes, part));
    }
}
