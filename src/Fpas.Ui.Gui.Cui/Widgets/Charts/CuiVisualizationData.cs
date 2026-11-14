namespace Fpas.Ui.Gui.Cui.Widgets.Charts;

/// <summary>A named nonnegative amount used by native visualizations.</summary>
/// <param name="Label">Literal caption.</param>
/// <param name="Value">Finite nonnegative amount.</param>
/// <param name="Color">Native palette selector or explicit RGBA.</param>
public sealed record CuiVizItem(string Label, float Value, CuiChartColor Color = default);

/// <summary>Native heatmap layout and scale.</summary>
/// <param name="RowLabels">Empty disables row labels; otherwise one per row. Empty entries omit a label.</param>
/// <param name="ColumnLabels">Empty disables column labels; otherwise one per column.</param>
/// <param name="Cell">Cell size; zero fits the available width.</param>
/// <param name="Minimum">Fixed range minimum, or equal to maximum for automatic.</param>
/// <param name="Maximum">Fixed range maximum.</param>
/// <param name="Color">Top color; automatic uses CUI's accent ramp.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
public sealed record CuiHeatmapOptions(string[]? RowLabels = null, string[]? ColumnLabels = null,
    float Cell = 0, float Minimum = 0, float Maximum = 0, CuiChartColor Color = default,
    CuiChartFormat? Format = null, int WidthKind = 0, float Width = 0);

/// <summary>Native calendar heatmap settings.</summary>
/// <param name="Cell">Day size; zero selects the native default.</param>
/// <param name="Color">Top color; automatic uses the native accent ramp.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
/// <param name="Unit">Optional literal unit; empty omits the total caption.</param>
public sealed record CuiCalendarHeatmapOptions(float Cell = 0, CuiChartColor Color = default,
    CuiChartFormat? Format = null, string Unit = "");

/// <summary>Native treemap dimensions and value format.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical height; zero uses the native default.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
public sealed record CuiTreemapOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    CuiChartFormat? Format = null);

/// <summary>Native treemap click and model-owned selection proposal.</summary>
/// <param name="Activated">Clicked input item index, otherwise minus one.</param>
/// <param name="Selected">Proposed selection, or minus one when absent.</param>
public readonly record struct CuiTreemapResult(int Activated, int Selected);

/// <summary>Native radar scale and presentation.</summary>
/// <param name="Size">Plot diameter; zero uses CUI's default.</param>
/// <param name="Maximum">Outer-ring value; zero selects an automatic range.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
/// <param name="Legend">Show native series captions.</param>
public sealed record CuiRadarOptions(float Size = 0, float Maximum = 0,
    CuiChartFormat? Format = null, bool Legend = true);

/// <summary>Native funnel presentation.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Color">Base color; automatic selects the native accent.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
public sealed record CuiFunnelOptions(int WidthKind = 1, float Width = 0,
    CuiChartColor Color = default, CuiChartFormat? Format = null);

/// <summary>Native split-bar presentation.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Bar height; zero uses the native default.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
/// <param name="Legend">Show the native wrapping legend.</param>
public sealed record CuiSplitBarOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    CuiChartFormat? Format = null, bool Legend = true);

/// <summary>Native waffle dimensions and formatting.</summary>
/// <param name="Cell">Square size; zero uses the native default.</param>
/// <param name="Columns">Squares per row, one through 60.</param>
/// <param name="Rows">Rows of squares, one through 60.</param>
/// <param name="Format">Safe numeric formatting, or native automatic.</param>
public sealed record CuiWaffleOptions(float Cell = 0, int Columns = 10, int Rows = 10,
    CuiChartFormat? Format = null);
