using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A named Sankey node with an optional native color.</summary>
/// <param name="Label">Literal caption.</param>
/// <param name="Color">Native palette or explicit RGBA.</param>
public sealed record CuiSankeyNode(string Label, CuiChartColor Color = default);

/// <summary>One directed flow; a zero amount hides the link.</summary>
/// <param name="From">Source input node index.</param>
/// <param name="To">Destination input node index, different from source.</param>
/// <param name="Value">Finite nonnegative flow.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiSankeyLink(int From, int To, float Value);

/// <summary>Native Sankey dimensions and formatting.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical height; zero selects the native default.</param>
/// <param name="NodeWidth">Node bar width; zero selects the native default.</param>
/// <param name="Format">Safe numeric formatting.</param>
public sealed record CuiSankeyOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    float NodeWidth = 0, CuiChartFormat? Format = null);

/// <summary>Native ranked bar-list presentation.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Format">Safe numeric formatting.</param>
/// <param name="MaximumRows">Zero shows all rows.</param>
/// <param name="KeepOrder">Retain input order instead of sorting by amount.</param>
/// <param name="Color">Override all item colors; automatic keeps native item colors.</param>
public sealed record CuiBarListOptions(int WidthKind = 1, float Width = 0, CuiChartFormat? Format = null,
    int MaximumRows = 0, bool KeepOrder = false, CuiChartColor Color = default);

/// <summary>Native concentric activity rings.</summary>
/// <param name="Size">Logical diameter; zero selects the native default.</param>
/// <param name="Thickness">Ring thickness; zero selects the native default.</param>
/// <param name="Center">Literal center readout.</param>
/// <param name="Caption">Literal caption below the center readout.</param>
/// <param name="Legend">Show the native side legend.</param>
public sealed record CuiActivityRingsOptions(float Size = 0, float Thickness = 0, string Center = "",
    string Caption = "", bool Legend = false);

/// <summary>A point in native scatter data units.</summary>
/// <param name="X">Horizontal coordinate.</param>
/// <param name="Y">Vertical coordinate.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiPlotPoint(float X, float Y);

/// <summary>One scatter or bubble series.</summary>
/// <param name="Name">Literal legend caption.</param>
/// <param name="Points">Finite input coordinates.</param>
/// <param name="Sizes">Null or empty for dots; otherwise a nonnegative size per point.</param>
/// <param name="Color">Native palette or explicit RGBA.</param>
public sealed record CuiScatterSeries(string Name, CuiPlotPoint[] Points, float[]? Sizes = null, CuiChartColor Color = default);

/// <summary>Native scatter axes, dimensions and legend.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical height; zero selects the native default.</param>
/// <param name="XLabel">Literal horizontal axis title.</param>
/// <param name="YLabel">Literal vertical axis title.</param>
/// <param name="XFormat">Safe horizontal value format.</param>
/// <param name="YFormat">Safe vertical value format.</param>
/// <param name="Legend">Show the native series legend.</param>
public sealed record CuiScatterOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    string XLabel = "", string YLabel = "", CuiChartFormat? XFormat = null, CuiChartFormat? YFormat = null, bool Legend = true);

/// <summary>A native gauge threshold and its semantic tone.</summary>
/// <param name="To">Increasing upper threshold.</param>
/// <param name="Tone">Native semantic tone, zero through eight.</param>
/// <param name="Name">Literal status chip caption.</param>
public sealed record CuiGaugeBand(float To, int Tone, string Name = "");

/// <summary>Native radial gauge styles.</summary>
public enum CuiGaugeStyle
{
    /// <summary>Segments when bands exist, otherwise an arc.</summary>
    Automatic,
    /// <summary>An arc with optional target and limit captions.</summary>
    Arc,
    /// <summary>Colored short segments with a status chip.</summary>
    Segments,
    /// <summary>A full ring with an optional detail caption.</summary>
    Ring
}

/// <summary>Native radial-gauge presentation.</summary>
/// <param name="Label">Literal caption below the value.</param>
/// <param name="Unit">Literal unit after the value.</param>
/// <param name="Format">Safe numeric format.</param>
/// <param name="Size">Logical diameter; zero selects the native default.</param>
/// <param name="Sweep">Arc degrees, zero for default or 90 through 300.</param>
/// <param name="Bands">At most sixteen increasing thresholds.</param>
/// <param name="ShowLimits">Show arc minimum and maximum captions.</param>
/// <param name="Target">Optional target marker.</param>
/// <param name="Style">Native rendering style.</param>
/// <param name="Detail">Literal detail below the gauge.</param>
/// <param name="Color">Optional native value color.</param>
public sealed record CuiRadialGaugeOptions(string Label = "", string Unit = "", CuiChartFormat? Format = null,
    float Size = 0, float Sweep = 0, CuiGaugeBand[]? Bands = null, bool ShowLimits = true,
    float? Target = null, CuiGaugeStyle Style = CuiGaugeStyle.Automatic, string Detail = "", CuiChartColor Color = default);

/// <summary>One native OHLC period and its nonnegative volume.</summary>
/// <param name="Open">Opening price.</param>
/// <param name="High">Highest price.</param>
/// <param name="Low">Lowest price.</param>
/// <param name="Close">Closing price.</param>
/// <param name="Volume">Nonnegative volume, zero for none.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiCandle(float Open, float High, float Low, float Close, float Volume = 0);

/// <summary>Native candlestick/OHLC chart presentation; CUI retains the viewport.</summary>
/// <param name="WidthKind">Native bridge width kind.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Height">Logical height; zero selects the native default.</param>
/// <param name="Labels">Empty or one literal time caption per candle.</param>
/// <param name="Format">Safe price format; null keeps CUI's two decimal places.</param>
/// <param name="Overlays">At most four line overlays; NaN samples are gaps.</param>
/// <param name="Volume">Show native volume bars when any candle has volume.</param>
/// <param name="Ohlc">Draw OHLC bars instead of candle bodies.</param>
/// <param name="Up">Optional native rising-period color.</param>
/// <param name="Down">Optional native falling-period color.</param>
public sealed record CuiCandlestickOptions(int WidthKind = 1, float Width = 0, float Height = 0,
    string[]? Labels = null, CuiChartFormat? Format = null, CuiChartSeries[]? Overlays = null,
    bool Volume = true, bool Ohlc = false, CuiChartColor Up = default, CuiChartColor Down = default);
