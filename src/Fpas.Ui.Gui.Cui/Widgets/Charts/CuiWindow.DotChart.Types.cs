using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Charts;

/// <summary>Native dot-matrix chart options.</summary>
/// <param name="Chart">Shared dimensions, ranges, formatting and annotations.</param>
/// <param name="Minimum">First X coordinate.</param>
/// <param name="Maximum">Last X coordinate.</param>
/// <param name="Step">Axis label spacing, or zero for automatic spacing.</param>
/// <param name="Units">Numeric labels or hours of the day.</param>
/// <param name="XFormat">Optional numeric axis format.</param>
/// <param name="Unit">Value unit.</param>
/// <param name="Pitch">Distance between dots, or zero for the default.</param>
/// <param name="Axis">Whether to show the axis labels.</param>
/// <param name="Readout">Whether to show the interactive value label.</param>
public sealed record CuiDotOptions(CuiChartOptions Chart, double Minimum, double Maximum, double Step,
    int Units, CuiChartFormat? XFormat, string Unit, float Pitch, bool Axis, bool Readout);

/// <summary>Native activation and readout.</summary>
/// <param name="Clicked">Whether the pointer or keyboard activated the chart.</param>
/// <param name="Index">Sample index, or minus one.</param>
/// <param name="X">X coordinate in axis units.</param>
/// <param name="Value">Stacked value at the sample.</param>
public readonly record struct CuiDotHit(bool Clicked, int Index, double X, float Value);
