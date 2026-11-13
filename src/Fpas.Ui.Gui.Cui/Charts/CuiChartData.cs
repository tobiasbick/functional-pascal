namespace Fpas.Ui.Gui.Cui;

/// <summary>Safe numeric formatting; literals are never interpreted as printf directives.</summary>
/// <param name="DecimalPlaces">Digits after the decimal point, zero through six.</param>
/// <param name="Prefix">Literal text before the number.</param>
/// <param name="Suffix">Literal text after the number.</param>
public sealed record CuiChartFormat(int DecimalPlaces = 0, string Prefix = "", string Suffix = "");

/// <summary>A native palette entry or an explicit straight-alpha color.</summary>
/// <param name="Palette">Zero for automatic, one through six for a theme series, minus one for RGBA.</param>
/// <param name="Rgba">Red in the low byte; used only for an explicit color.</param>
public readonly record struct CuiChartColor(int Palette = 0, uint Rgba = 0);

/// <summary>One named series of finite samples; differing sample counts are supported.</summary>
/// <param name="Name">Legend caption.</param>
/// <param name="Values">Samples, copied before the call returns.</param>
/// <param name="Color">Automatic, palette or explicit color.</param>
public sealed record CuiChartSeries(string Name, float[] Values, CuiChartColor Color = default);

/// <summary>A native axis or point annotation.</summary>
/// <param name="Kind">X axis zero, Y axis one, point two.</param>
/// <param name="X">Sample-index coordinate.</param>
/// <param name="Y">Value coordinate.</param>
/// <param name="Label">Literal caption.</param>
/// <param name="Detail">Literal second line; empty omits it.</param>
/// <param name="Accent">Use the accent color.</param>
public sealed record CuiChartAnnotation(int Kind, double X, double Y, string Label = "", string Detail = "", bool Accent = false);

/// <summary>Native line and bar options, with automatic bounds when minimum equals maximum.</summary>
/// <param name="WidthKind">Content zero, fill one, fixed two, percentage three.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical plot height; zero selects CUI's default.</param>
/// <param name="Labels">Optional sample captions; missing entries fall back to native index captions.</param>
/// <param name="Format">Null selects CUI's automatic number formatting.</param>
/// <param name="Area">Fill a line chart.</param>
/// <param name="Grid">Draw native grid lines.</param>
/// <param name="Legend">Show native series captions.</param>
/// <param name="Stacked">Stack series.</param>
/// <param name="Smooth">Smooth a line chart.</param>
/// <param name="Dither">Dither area fills.</param>
/// <param name="Minimum">Fixed lower value bound.</param>
/// <param name="Maximum">Fixed upper value bound.</param>
/// <param name="Annotations">Native reference lines and points.</param>
public sealed record CuiChartOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    string[]? Labels = null, CuiChartFormat? Format = null, bool Area = false, bool Grid = true,
    bool Legend = true, bool Stacked = false, bool Smooth = false, bool Dither = false,
    float Minimum = 0, float Maximum = 0, CuiChartAnnotation[]? Annotations = null);

/// <summary>One native pattern-meter part.</summary>
/// <param name="Name">Literal legend caption.</param>
/// <param name="Value">Nonnegative amount.</param>
/// <param name="Color">Automatic, palette or explicit color.</param>
/// <param name="Density">Zero chooses the native sequence; otherwise a fraction through one.</param>
public sealed record CuiMeterPart(string Name, float Value, CuiChartColor Color = default, float Density = 0);

/// <summary>Native pattern-meter presentation.</summary>
/// <param name="Total">Zero uses the sum; a positive capacity includes free space.</param>
/// <param name="Label">Literal header.</param>
/// <param name="Unit">Literal amount suffix.</param>
/// <param name="Format">Optional safe amount formatting.</param>
/// <param name="FreeLabel">Null uses CUI's default; empty hides free space in the legend.</param>
/// <param name="Grayscale">Use foreground dots rather than series colors.</param>
/// <param name="Legend">Show the native wrapping legend.</param>
/// <param name="Height">Bar height; zero uses the native default.</param>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
public sealed record CuiMeterOptions(float Total = 0, string Label = "", string Unit = "",
    CuiChartFormat? Format = null, string? FreeLabel = null, bool Grayscale = false,
    bool Legend = true, float Height = 0, int WidthKind = 1, float Width = 0);
