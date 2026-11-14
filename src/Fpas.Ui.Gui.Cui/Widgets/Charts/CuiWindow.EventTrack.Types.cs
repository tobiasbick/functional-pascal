using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Charts;

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
