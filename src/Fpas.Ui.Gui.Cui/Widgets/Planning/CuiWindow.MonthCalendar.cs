using Fpas.Ui.Gui.Cui.Interop.Planning;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Planning;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native month calendar.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="events">Current appointments.</param>
    /// <param name="flags">Native presentation and input flags.</param>
    /// <param name="focus">Requested native part.</param>
    /// <param name="part">Tracks visible native parts.</param>
    /// <param name="month">Displayed month day number.</param>
    /// <param name="selected">Selected day number or minus one.</param>
    /// <param name="firstWeekday">First weekday index.</param>
    /// <param name="cellHeight">Native day-cell height.</param>
    /// <param name="months">Optional localized month names.</param>
    /// <param name="days">Day count or localized weekday names.</param>
    /// <param name="today">Current day number or timeline offset.</param>
    /// <returns>Native model proposal.</returns>
    public unsafe CuiMonthResult MonthCalendar(string id, int month, int selected, CuiMonthEvent[] events,
        int firstWeekday, float cellHeight, string[] months, string[] days, int today,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        if (months.Length is not (0 or 12) || days.Length is not (0 or 7))
            throw new ArgumentException("Expected twelve month names and seven weekday names.");
        using var memory = new ChartMemory();
        var data = new NativeMonthEvent[events.Length];
        for (int i = 0; i < data.Length; ++i)
        {
            var e = events[i];
            Utf8.Validate(e.Title, nameof(events));
            data[i] = new() { Title = memory.Text(e.Title), Start = e.Start, Finish = e.Finish,
                Time = e.Time, AllDay = e.AllDay ? 1 : 0, Tone = e.Tone };
        }
        foreach (string label in months) Utf8.Validate(label, nameof(months));
        foreach (string label in days) Utf8.Validate(label, nameof(days));
        var nodes = new int[4];
        int result, activated;
        fixed (NativeMonthEvent* values = data)
        fixed (int* parts = nodes)
            result = NativeMethods.MonthCalendar(context, Stable(id), ref month, ref selected, values, data.Length,
                firstWeekday, cellHeight, memory.Labels(months), memory.Labels(days), today, flags, focus, parts, out activated);
        return new(month, selected, activated, result, TrackScheduleParts(nodes, part));
    }
}
