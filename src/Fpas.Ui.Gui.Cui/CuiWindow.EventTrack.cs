using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native event-track options.</summary>
/// <param name="WidthKind">Encoded width extent.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Capacity">Maximum event count.</param>
/// <param name="Minimum">First axis coordinate.</param>
/// <param name="Maximum">Last axis coordinate.</param>
/// <param name="Step">Event snapping distance, or zero.</param>
/// <param name="Units">Numeric labels or hours of the day.</param>
/// <param name="Format">Optional numeric axis format.</param>
/// <param name="LabelStep">Axis label spacing, or zero for automatic spacing.</param>
/// <param name="Add">Whether to accept insertions.</param>
/// <param name="Remove">Whether to accept removals.</param>
/// <param name="Axis">Whether to show axis labels.</param>
public sealed record CuiEventOptions(int WidthKind, float Width, int Capacity, double Minimum,
    double Maximum, double Step, int Units, CuiChartFormat? Format, double LabelStep,
    bool Add, bool Remove, bool Axis);

/// <summary>Native event track result.</summary>
/// <param name="Changed">Whether the native track edited a time.</param>
/// <param name="Times">Proposed events in time order.</param>
/// <param name="Selected">Selected index, or minus one.</param>
/// <param name="Added">New event index, or minus one.</param>
/// <param name="Removed">Former event index, or minus one.</param>
/// <param name="Dragging">Whether an event is being dragged.</param>
public readonly record struct CuiEventResult(bool Changed, double[] Times, int Selected, int Added,
    int Removed, bool Dragging);

public sealed partial class CuiWindow
{
    /// <summary>Draws native editable time markers and returns a proposal.</summary>
    /// <param name="id">Stable track identity.</param>
    /// <param name="events">Current sorted events.</param>
    /// <param name="selected">Current selected index, or minus one.</param>
    /// <param name="options">Native bounds, capacity and input settings.</param>
    /// <param name="enabled">Whether the track accepts input.</param>
    /// <param name="focus">Whether to focus the track.</param>
    /// <returns>Proposed markers, selection, and edit details.</returns>
    public unsafe CuiEventResult EventTrack(string id, double[] events, int selected,
        CuiEventOptions options, bool enabled, bool focus)
    {
        CheckFrame();
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        if (options.Capacity < events.Length || selected < -1 || selected >= events.Length ||
            !double.IsFinite(options.Minimum) || !double.IsFinite(options.Maximum - options.Minimum) ||
            options.Maximum <= options.Minimum || !double.IsFinite(options.Step) || options.Step < 0 ||
            !double.IsFinite(options.LabelStep) || options.LabelStep < 0 || options.Units is < 0 or > 1 ||
            options.Units == 1 && (Math.Abs(options.Minimum) > 35_000_000 || Math.Abs(options.Maximum) > 35_000_000))
            throw new ArgumentException("Invalid event track model or axis.", nameof(options));
        double previous = options.Minimum;
        foreach (double time in events)
        {
            if (!double.IsFinite(time) || time < previous || time > options.Maximum)
                throw new ArgumentException("Event times must be finite, sorted and on the axis.", nameof(events));
            previous = time;
        }
        var times = new double[Math.Max(1, options.Capacity)];
        events.CopyTo(times, 0);
        using var memory = new ChartMemory();
        var native = new NativeEventOptions {
            Minimum = options.Minimum, Maximum = options.Maximum, Step = options.Step,
            LabelStep = options.LabelStep, Format = memory.Format(options.Format), Width = options.Width,
            WidthKind = options.WidthKind, Units = options.Units,
            Flags = (options.Add ? 0 : 1) | (options.Remove ? 0 : 2) | (options.Axis ? 0 : 4) |
                (enabled ? 8 : 0) | (focus ? 16 : 0)
        };
        int count = events.Length;
        int* result = stackalloc int[5];
        result[0] = selected;
        fixed (double* values = times)
            NativeMethods.EventTrack(context, Stable(id), values, ref count, options.Capacity, in native, result);
        return new(result[4] != 0, times.AsSpan(0, count).ToArray(), result[0], result[1], result[2], result[3] != 0);
    }
}
