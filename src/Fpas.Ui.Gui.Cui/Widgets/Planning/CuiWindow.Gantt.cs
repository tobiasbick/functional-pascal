// [Documentation](../../../../docs/ui/gui/planning/gantt.md)
using Fpas.Ui.Gui.Cui.Interop.Planning;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Planning;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Gantt chart.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="heightKind">Native height kind.</param>
    /// <param name="height">Fixed height or percentage of available height.</param>
    /// <param name="flags">Native presentation and input flags.</param>
    /// <param name="focus">Requested native part.</param>
    /// <param name="part">Tracks visible native parts.</param>
    /// <param name="days">Number of days on the timeline.</param>
    /// <param name="today">Timeline offset of today, or zero to hide the marker.</param>
    /// <param name="origin">First timeline day number.</param>
    /// <param name="tasks">Current timeline tasks.</param>
    /// <param name="dayWidth">Native timeline day width.</param>
    /// <param name="rowHeight">Native task row height.</param>
    /// <returns>Native model proposal.</returns>
    public unsafe CuiGanttResult Gantt(string id, int origin, int days, CuiGanttTask[] tasks, int today,
        float dayWidth, float rowHeight, int heightKind, float height, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckExtent(heightKind, height, nameof(height));
        if (origin < DateOnly.MinValue.DayNumber || days <= 0 ||
            (long)origin + days - 1 > DateOnly.MaxValue.DayNumber)
            throw new ArgumentOutOfRangeException(nameof(days), "The timeline must cover a representable date range.");
        if (today < 0 || today >= days) throw new ArgumentOutOfRangeException(nameof(today));
        ArgumentNullException.ThrowIfNull(tasks);
        CheckFeedbackNumber(dayWidth, nameof(dayWidth));
        CheckFeedbackNumber(rowHeight, nameof(rowHeight));
        if (!float.IsFinite(dayWidth * days)) throw new ArgumentOutOfRangeException(nameof(dayWidth));
        if (rowHeight is > 0 and <= 16 || !float.IsFinite(rowHeight * tasks.Length))
            throw new ArgumentOutOfRangeException(nameof(rowHeight));
        using var memory = new ChartMemory();
        var data = new NativeGanttTask[tasks.Length];
        for (int i = 0; i < data.Length; ++i)
        {
            var t = tasks[i];
            Utf8.Validate(t.Name, nameof(tasks));
            if (t.Owner is not null) Utf8.Validate(t.Owner, nameof(tasks));
            if (t.Start < 0 || t.Finish > days || t.Finish < t.Start || t.Finish == t.Start && !t.Milestone ||
                !float.IsFinite(t.Progress) || t.Progress is < 0 or > 1 || t.State is < 0 or > 3 ||
                t.Tone is < 0 or > 8 || t.Dependency < -1 || t.Dependency >= tasks.Length)
                throw new ArgumentException("Invalid task dates, progress, dependency, tone or state.", nameof(tasks));
            data[i] = new() { Name = memory.Text(t.Name), Owner = memory.Text(t.Owner),
                Start = t.Start, Finish = t.Finish, Progress = t.Progress, Tone = t.Tone,
                Dependency = t.Dependency, Milestone = t.Milestone ? 1 : 0, State = t.State };
        }
        var nodes = new int[data.Length];
        int activated;
        fixed (NativeGanttTask* values = data)
        fixed (int* parts = nodes)
            activated = NativeMethods.Gantt(context, Stable(id), origin, days, values, data.Length,
                today, dayWidth, rowHeight, heightKind, height, flags, focus, parts);
        CuiGanttTask[]? edited = null;
        for (int i = 0; i < data.Length; ++i)
        {
            var t = tasks[i];
            var value = data[i];
            if (t.Start != value.Start || t.Finish != value.Finish)
            {
                edited ??= (CuiGanttTask[])tasks.Clone();
                edited[i] = t with { Start = value.Start, Finish = value.Finish };
            }
        }
        return new(edited, activated, TrackScheduleParts(nodes, part));
    }
}
