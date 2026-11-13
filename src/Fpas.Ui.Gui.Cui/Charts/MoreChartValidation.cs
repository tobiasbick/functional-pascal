using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static void CheckSankey(CuiSankeyNode[] nodes, CuiSankeyLink[] links, CuiSankeyOptions options)
    {
        ArgumentNullException.ThrowIfNull(nodes); ArgumentNullException.ThrowIfNull(links);
        if ((long)nodes.Length + links.Length > int.MaxValue) throw new ArgumentOutOfRangeException(nameof(links));
        foreach (var node in nodes) { ArgumentNullException.ThrowIfNull(node); Utf8.Validate(node.Label, nameof(nodes)); CheckChartColor(node.Color); }
        float total = 0;
        foreach (var link in links)
        {
            if (link.From < 0 || link.From >= nodes.Length || link.To < 0 || link.To >= nodes.Length || link.From == link.To)
                throw new ArgumentOutOfRangeException(nameof(links));
            CheckVizDistance(link.Value); total += link.Value;
        }
        if (!float.IsFinite(total)) throw new ArgumentOutOfRangeException(nameof(links));
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        CheckVizDistance(options.Height); CheckVizDistance(options.NodeWidth); CheckChartFormat(options.Format);
    }

    private static void CheckBarList(CuiVizItem[] items, CuiBarListOptions options)
    {
        CheckVizItems(items);
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (var item in items)
            if (!names.Add(item.Label)) throw new ArgumentException("Bar-list labels must be unique native identities.", nameof(items));
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckChartFormat(options.Format); CheckChartColor(options.Color);
        if (options.MaximumRows < 0) throw new ArgumentOutOfRangeException(nameof(options));
    }

    private static void CheckScatter(CuiScatterSeries[] series, CuiScatterOptions options)
    {
        ArgumentNullException.ThrowIfNull(series);
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckVizDistance(options.Height);
        Utf8.Validate(options.XLabel, nameof(options)); Utf8.Validate(options.YLabel, nameof(options));
        CheckChartFormat(options.XFormat); CheckChartFormat(options.YFormat);
        float xl = 1e30f, xh = -1e30f, yl = 1e30f, yh = -1e30f; long count = 0;
        foreach (var item in series)
        {
            ArgumentNullException.ThrowIfNull(item); Utf8.Validate(item.Name, nameof(series));
            ArgumentNullException.ThrowIfNull(item.Points); CheckChartColor(item.Color);
            count += item.Points.Length;
            if (item.Sizes is { Length: > 0 } sizes)
            {
                if (sizes.Length != item.Points.Length) throw new ArgumentException("Bubble sizes must match the points.", nameof(series));
                CheckVizSamples(sizes, true);
            }
            foreach (var point in item.Points)
            {
                if (!float.IsFinite(point.X) || !float.IsFinite(point.Y)) throw new ArgumentOutOfRangeException(nameof(series));
                xl = Math.Min(xl, point.X); xh = Math.Max(xh, point.X); yl = Math.Min(yl, point.Y); yh = Math.Max(yh, point.Y);
            }
        }
        if (count > int.MaxValue) throw new ArgumentOutOfRangeException(nameof(series));
        if (count > 0)
        {
            float px = (xh - xl) * .015f, py = (yh - yl) * .03f;
            CheckNativeAxis(xl - px, xh + px, 5); CheckNativeAxis(yl - py, yh + py, 4);
        }
    }

    private static void CheckRadialGauge(float value, float minimum, float maximum, CuiRadialGaugeOptions options)
    {
        if (!float.IsFinite(value) || !float.IsFinite(minimum) || !float.IsFinite(maximum) || maximum <= minimum ||
            !float.IsFinite(maximum - minimum) || !float.IsFinite(value - minimum)) throw new ArgumentOutOfRangeException(nameof(value));
        foreach (string text in new[] { options.Label, options.Unit, options.Detail }) Utf8.Validate(text, nameof(options));
        CheckVizDistance(options.Size); CheckVizDistance(options.Sweep); CheckChartFormat(options.Format); CheckChartColor(options.Color);
        if (options.Sweep != 0 && (options.Sweep < 90 || options.Sweep > 300) || !Enum.IsDefined(options.Style))
            throw new ArgumentOutOfRangeException(nameof(options));
        if (options.Target is { } target && (!float.IsFinite(target) || !float.IsFinite((target - minimum) / (maximum - minimum))))
            throw new ArgumentOutOfRangeException(nameof(options));
        var bands = options.Bands ?? [];
        if (bands.Length > 16) throw new ArgumentOutOfRangeException(nameof(options));
        float previous = minimum;
        foreach (var band in bands)
        {
            ArgumentNullException.ThrowIfNull(band); Utf8.Validate(band.Name, nameof(options));
            if (!float.IsFinite(band.To) || band.To <= previous || band.To > maximum || band.Tone is < 0 or > 8)
                throw new ArgumentOutOfRangeException(nameof(options));
            previous = band.To;
        }
    }

    private static void CheckCandles(CuiCandle[] candles, CuiCandlestickOptions options)
    {
        ArgumentNullException.ThrowIfNull(candles);
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckVizDistance(options.Height);
        CheckVizLabels(options.Labels, candles.Length); CheckChartFormat(options.Format); CheckChartColor(options.Up); CheckChartColor(options.Down);
        float volume = 0;
        foreach (var candle in candles)
        {
            if (!float.IsFinite(candle.Open) || !float.IsFinite(candle.High) || !float.IsFinite(candle.Low) ||
                !float.IsFinite(candle.Close) || candle.Low > Math.Min(candle.Open, candle.Close) ||
                candle.High < Math.Max(candle.Open, candle.Close) || !float.IsFinite(candle.High - candle.Low))
                throw new ArgumentOutOfRangeException(nameof(candles));
            CheckVizDistance(candle.Volume); volume += candle.Volume;
            // CUI-019 (docs/future/cui-upstream-bugs.md#cui-019): reject collapsed price expansions.
            if (candle.High - candle.Low < 1e-6f && (candle.High + .5f <= candle.High || candle.Low - .5f >= candle.Low))
                throw new ArgumentOutOfRangeException(nameof(candles));
        }
        if (!float.IsFinite(volume)) throw new ArgumentOutOfRangeException(nameof(candles));
        var overlays = options.Overlays ?? [];
        if (overlays.Length > 4) throw new ArgumentOutOfRangeException(nameof(options));
        foreach (var overlay in overlays)
        {
            ArgumentNullException.ThrowIfNull(overlay); Utf8.Validate(overlay.Name, nameof(options));
            ArgumentNullException.ThrowIfNull(overlay.Values); CheckChartColor(overlay.Color);
            if (overlay.Values.Length > candles.Length) throw new ArgumentException("An overlay cannot outgrow its candle series.", nameof(options));
            foreach (float sample in overlay.Values)
                if (float.IsInfinity(sample)) throw new ArgumentOutOfRangeException(nameof(options));
        }
    }
}
