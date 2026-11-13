using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

internal sealed unsafe partial class ChartMemory
{
    internal NativeSankeyNode* Nodes(CuiSankeyNode[] nodes)
    {
        var data = new NativeSankeyNode[nodes.Length];
        for (int i = 0; i < data.Length; i++)
            data[i] = new() { Label = Text(nodes[i].Label), Palette = nodes[i].Color.Palette, Rgba = nodes[i].Color.Rgba };
        return Copy<NativeSankeyNode>(data);
    }

    internal NativeScatterSeries* Scatter(CuiScatterSeries[] series)
    {
        var data = new NativeScatterSeries[series.Length];
        for (int i = 0; i < data.Length; i++)
            data[i] = new() { Name = Text(series[i].Name), Points = Copy<CuiPlotPoint>(series[i].Points),
                Sizes = series[i].Sizes is { Length: > 0 } sizes ? Copy<float>(sizes) : null,
                Count = series[i].Points.Length, Palette = series[i].Color.Palette, Rgba = series[i].Color.Rgba };
        return Copy<NativeScatterSeries>(data);
    }

    internal NativeRadialOptions Options(CuiRadialGaugeOptions options)
    {
        var bands = options.Bands ?? [];
        var data = new NativeGaugeBand[bands.Length];
        for (int i = 0; i < data.Length; i++)
            data[i] = new() { To = bands[i].To, Tone = bands[i].Tone, Name = Text(bands[i].Name) };
        return new() { Label = Text(options.Label), Unit = Text(options.Unit), Format = Format(options.Format),
            Size = options.Size, Sweep = options.Sweep, Bands = Copy<NativeGaugeBand>(data), BandCount = data.Length,
            Flags = (options.ShowLimits ? 1 : 0) | (options.Target.HasValue ? 2 : 0), Target = options.Target ?? 0,
            Style = (int)options.Style, Detail = Text(options.Detail), Palette = options.Color.Palette, Rgba = options.Color.Rgba };
    }

    internal NativeCandleOptions Options(CuiCandlestickOptions options) => new() {
        WidthKind = options.WidthKind, Width = options.Width, Height = options.Height,
        Labels = Labels(options.Labels), Format = Format(options.Format),
        Overlays = options.Overlays is { } overlays ? Series(overlays) : null,
        OverlayCount = options.Overlays?.Length ?? 0, Flags = (!options.Volume ? 1 : 0) | (options.Ohlc ? 2 : 0),
        UpPalette = options.Up.Palette, UpRgba = options.Up.Rgba, DownPalette = options.Down.Palette, DownRgba = options.Down.Rgba };
}
