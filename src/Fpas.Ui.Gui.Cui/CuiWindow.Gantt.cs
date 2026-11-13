using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native timeline task.</summary>
/// <param name="Name">Task name.</param>
/// <param name="Owner">Optional owner.</param>
/// <param name="Start">Start offset.</param>
/// <param name="Finish">End offset.</param>
/// <param name="Progress">Completed fraction.</param>
/// <param name="Tone">Semantic color.</param>
/// <param name="Dependency">Dependency index or minus one.</param>
/// <param name="Milestone">Milestone appearance.</param>
/// <param name="State">Native task state.</param>
public readonly record struct CuiGanttTask(string Name, string Owner, int Start, int Finish, float Progress, int Tone, int Dependency, bool Milestone, int State);

/// <summary>Native edit proposal and tracked parts.</summary>
/// <param name="Tasks">Changed task schedule, or null.</param>
/// <param name="Activated">Activated task index.</param>
/// <param name="Parts">Visible task availability.</param>
public readonly record struct CuiGanttResult(CuiGanttTask[]? Tasks, int Activated, bool[] Parts);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Gantt chart.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="heightKind">Native height kind.</param>
    /// <param name="height">Preferred height.</param>
    /// <param name="flags">Native presentation and input flags.</param>
    /// <param name="focus">Requested native part.</param>
    /// <param name="part">Tracks visible native parts.</param>
    /// <param name="days">Day count or localized weekday names.</param>
    /// <param name="today">Current day number or timeline offset.</param>
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
        using var memory = new ChartMemory();
        var data = new NativeGanttTask[tasks.Length];
        for (int i = 0; i < data.Length; ++i)
        {
            var t = tasks[i];
            Utf8.Validate(t.Name, nameof(tasks));
            Utf8.Validate(t.Owner, nameof(tasks));
            data[i] = new() { Name = memory.Text(t.Name), Owner = memory.Text(Optional(t.Owner)),
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
