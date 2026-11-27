// [Documentation](../../../../docs/ui/gui/planning/month-calendar.md)
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
    /// <param name="month">Displayed .NET day number, or minus one for the current month.</param>
    /// <param name="selected">Selected .NET day number or minus one.</param>
    /// <param name="firstWeekday">First weekday index, from Monday zero to Sunday six.</param>
    /// <param name="cellHeight">Native day-cell height.</param>
    /// <param name="months">Optional localized month names.</param>
    /// <param name="days">Optional localized weekday names, Monday first.</param>
    /// <param name="today">Override .NET day number, or minus one for the local system date.</param>
    /// <returns>Native model proposal.</returns>
    public unsafe CuiMonthResult MonthCalendar(string id, int month, int selected, CuiMonthEvent[] events,
        int firstWeekday, float cellHeight, string[] months, string[] days, int today,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(events);
        ArgumentNullException.ThrowIfNull(months);
        ArgumentNullException.ThrowIfNull(days);
        var last = DateOnly.MaxValue.DayNumber;
        if (month < -1 || month > last) throw new ArgumentOutOfRangeException(nameof(month));
        if (selected < -1 || selected > last) throw new ArgumentOutOfRangeException(nameof(selected));
        if (today < -1 || today > last) throw new ArgumentOutOfRangeException(nameof(today));
        if (firstWeekday is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(firstWeekday));
        if (focus is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(focus));
        CheckFeedbackNumber(cellHeight, nameof(cellHeight));
        if (months.Length is not (0 or 12) || days.Length is not (0 or 7))
            throw new ArgumentException("Expected twelve month names and seven weekday names.");
        using var memory = new ChartMemory();
        var data = new NativeMonthEvent[events.Length];
        for (int i = 0; i < data.Length; ++i)
        {
            var e = events[i];
            Utf8.Validate(e.Title, nameof(events));
            CheckTone(e.Tone);
            if (e.Start < 0 || e.Start > last || e.Finish < -1 || e.Finish > last ||
                !float.IsFinite(e.Time) || e.Time is < 0 or > 24)
                throw new ArgumentException("A month appointment needs representable dates and a valid hour.", nameof(events));
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
