// [Documentation](../../../../docs/ui/gui/charts/README.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Charts;

/// <summary>Floating-point notation understood by CUI.</summary>
public enum CuiValueNotation : byte
{
    /// <summary>Fixed decimal notation.</summary>
    Fixed,
    /// <summary>Decimal notation with an exponent.</summary>
    Scientific,
    /// <summary>Fixed or scientific notation based on the value and precision.</summary>
    General,
    /// <summary>Hexadecimal notation with a binary exponent.</summary>
    Hexadecimal
}

/// <summary>How the native number format signals nonnegative values.</summary>
public enum CuiValueSign : byte
{
    /// <summary>Only negative values have a sign.</summary>
    NegativeOnly,
    /// <summary>Nonnegative values have a plus sign.</summary>
    Always,
    /// <summary>Nonnegative values have a leading space.</summary>
    Space
}

/// <summary>How the native number format fills the minimum field width.</summary>
public enum CuiValuePadding : byte
{
    /// <summary>Add spaces before the value.</summary>
    LeadingSpaces,
    /// <summary>Add spaces after the value.</summary>
    TrailingSpaces,
    /// <summary>Add zeros after the sign and numeric prefix.</summary>
    LeadingZeros
}

/// <summary>Safe numeric formatting; literals are never interpreted as printf directives.</summary>
/// <param name="DecimalPlaces">Precision from zero to 999. Null uses the notation's native default.</param>
/// <param name="Prefix">Literal text before the number.</param>
/// <param name="Suffix">Literal text after the number.</param>
/// <param name="Notation">Fixed, scientific, general or hexadecimal notation.</param>
/// <param name="FieldWidth">Minimum numeric field width from zero to 999.</param>
/// <param name="Padding">How to fill unused field width.</param>
/// <param name="Sign">Sign for a nonnegative value.</param>
/// <param name="Alternate">Retain the decimal point and, for general notation, trailing zeros.</param>
/// <param name="Uppercase">Use uppercase exponent and hexadecimal letters.</param>
public sealed record CuiChartFormat(int? DecimalPlaces = 0, string Prefix = "", string Suffix = "",
    CuiValueNotation Notation = CuiValueNotation.Fixed, int FieldWidth = 0,
    CuiValuePadding Padding = CuiValuePadding.LeadingSpaces, CuiValueSign Sign = CuiValueSign.NegativeOnly,
    bool Alternate = false, bool Uppercase = false);

/// <summary>A native chart color from the theme or an explicit straight-alpha color.</summary>
/// <param name="Palette">Zero for automatic, one through six for a theme series, minus one for RGBA.</param>
/// <param name="Rgba">Red in the low byte; used only for an explicit color.</param>
public readonly record struct CuiChartColor(int Palette = 0, uint Rgba = 0)
{
    private const int FirstSeriesToken = 29;
    private const int SeriesCount = 6;

    internal bool IsValid => Palette >= -FirstSeriesToken && Palette <= SeriesCount;

    /// <summary>Selects a semantic color from CUI's active theme.</summary>
    /// <param name="token">CUI color token ordinal, from Background through Series6.</param>
    /// <returns>The theme selector for use by native widgets.</returns>
    public static CuiChartColor Theme(int token)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(token, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThanOrEqual(token, FirstSeriesToken + SeriesCount);
        return new(token >= FirstSeriesToken ? token - FirstSeriesToken + 1 : -token - 1);
    }
}

/// <summary>One named series of finite samples; differing sample counts are supported.</summary>
/// <param name="Name">Legend caption.</param>
/// <param name="Values">Samples, copied before the call returns.</param>
/// <param name="Color">Automatic, palette or explicit color.</param>
public sealed record CuiChartSeries(string Name, float[] Values, CuiChartColor Color = default);

/// <summary>A native axis or point annotation.</summary>
/// <param name="Kind">X axis zero, Y axis one, point two.</param>
/// <param name="X">Sample index or coordinate in the chosen X interval.</param>
/// <param name="Y">Value coordinate.</param>
/// <param name="Label">Literal caption, or null to omit it.</param>
/// <param name="Detail">Literal second line for vertical lines and points, or null to omit it.</param>
/// <param name="Accent">Use the accent color when Color is automatic.</param>
/// <param name="Color">Native theme or explicit color for the shape and caption.</param>
public sealed record CuiChartAnnotation(int Kind, double X, double Y,
    string? Label = null, string? Detail = null, bool Accent = false, CuiChartColor Color = default);

/// <summary>Native line and bar options, with automatic bounds when minimum equals maximum.</summary>
/// <param name="WidthKind">Content zero, fill one, fixed two, percentage three.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical plot height; zero selects CUI's default.</param>
/// <param name="Labels">Optional captions in sample order. Null or empty omits X-axis captions.</param>
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
/// <param name="XMinimum">X coordinate of the first sample. Equal X bounds select zero-based sample indices.</param>
/// <param name="XMaximum">X coordinate of the last sample. Use equal X bounds or an increasing interval.</param>
public sealed record CuiChartOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    string[]? Labels = null, CuiChartFormat? Format = null, bool Area = false, bool Grid = true,
    bool Legend = true, bool Stacked = false, bool Smooth = false, bool Dither = false,
    float Minimum = 0, float Maximum = 0, CuiChartAnnotation[]? Annotations = null,
    double XMinimum = 0, double XMaximum = 0);

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
