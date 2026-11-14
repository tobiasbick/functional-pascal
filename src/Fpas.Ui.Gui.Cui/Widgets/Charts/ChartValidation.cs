using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static void CheckChartFormat(CuiChartFormat? format)
    {
        if (format is null) return;
        Utf8.Validate(format.Prefix, nameof(format));
        Utf8.Validate(format.Suffix, nameof(format));
        if (format.DecimalPlaces is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(format));
    }

    private static void CheckChartColor(CuiChartColor color)
    {
        if (color.Palette is < -1 or > 6) throw new ArgumentOutOfRangeException(nameof(color));
    }

    private static void CheckChartOptions(CuiChartOptions options)
    {
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        if (!float.IsFinite(options.Height) || options.Height < 0 ||
            !float.IsFinite(options.Minimum) || !float.IsFinite(options.Maximum) ||
            options.Minimum > options.Maximum)
            throw new ArgumentOutOfRangeException(nameof(options));
        CheckChartFormat(options.Format);
        if (options.Labels is { } labels)
            foreach (string label in labels) Utf8.Validate(label, nameof(options));
        if (options.Annotations is { } annotations)
            foreach (var item in annotations)
            {
                ArgumentNullException.ThrowIfNull(item);
                if (item.Kind is < 0 or > 2 || !double.IsFinite(item.X) || !double.IsFinite(item.Y))
                    throw new ArgumentOutOfRangeException(nameof(options));
                Utf8.Validate(item.Label, nameof(options));
                Utf8.Validate(item.Detail, nameof(options));
            }
    }

    private static void CheckChartSeries(CuiChartSeries[] series, CuiChartOptions options, bool bars)
    {
        ArgumentNullException.ThrowIfNull(series);
        CheckChartOptions(options);
        int count = 0;
        foreach (var item in series)
        {
            ArgumentNullException.ThrowIfNull(item);
            Utf8.Validate(item.Name, nameof(series));
            ArgumentNullException.ThrowIfNull(item.Values);
            CheckChartColor(item.Color);
            foreach (float sample in item.Values)
                if (!float.IsFinite(sample)) throw new ArgumentOutOfRangeException(nameof(series));
            count = Math.Max(count, item.Values.Length);
        }
        if (count == 0) return;
        float lo = float.PositiveInfinity, hi = float.NegativeInfinity;
        for (int i = 0; i < count; i++)
        {
            float sum = 0;
            foreach (var item in series)
                if (i < item.Values.Length)
                {
                    if (options.Stacked)
                    {
                        sum += item.Values[i];
                        if (!float.IsFinite(sum)) throw new ArgumentOutOfRangeException(nameof(series));
                    }
                    else { lo = Math.Min(lo, item.Values[i]); hi = Math.Max(hi, item.Values[i]); }
                }
            if (options.Stacked) { lo = Math.Min(lo, sum); hi = Math.Max(hi, sum); }
        }
        if (options.Minimum != options.Maximum) { lo = options.Minimum; hi = options.Maximum; }
        else if (bars || lo >= 0 && lo <= hi * .45f) lo = Math.Min(lo, 0);
        CheckNativeAxis(lo, hi, 4);
    }

    private static void CheckNativeAxis(float lo, float hi, int intervals)
    {
        // CUI-019 (docs/future/cui-upstream-bugs.md#cui-019): reject non-advancing float ticks.
        if (hi <= lo) hi = lo + 1;
        float rough = (hi - lo) / intervals;
        float magnitude = MathF.Pow(10, MathF.Floor(MathF.Log10(rough)));
        float norm = rough / magnitude;
        float step = magnitude * (norm < 1.5f ? 1 : norm < 2.25f ? 2 : norm < 3.5f ? 2.5f : norm < 7.5f ? 5 : 10);
        float lower = MathF.Floor(lo / step) * step, upper = MathF.Ceiling(hi / step) * step;
        if (!float.IsFinite(lower) || !float.IsFinite(upper) || !float.IsFinite(step) || step <= 0 ||
            !float.IsFinite(upper - lower) || !float.IsFinite(upper + step) ||
            lower + step <= lower || upper + step <= upper || (upper - lower) / step > 32)
            throw new ArgumentOutOfRangeException(nameof(lo), "The native chart cannot represent this value range. Supply wider explicit bounds.");
    }

    private static void CheckChartValues(float[] values)
    {
        ArgumentNullException.ThrowIfNull(values);
        float total = 0;
        foreach (float value in values)
        {
            if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(nameof(values));
            total += value;
        }
        if (!float.IsFinite(total)) throw new ArgumentOutOfRangeException(nameof(values));
    }
}
