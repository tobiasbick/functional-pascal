// [Documentation](../../../../docs/ui/gui/charts/README.md)
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
        if (format.DecimalPlaces is < 0 or > 999 || format.FieldWidth is < 0 or > 999 ||
            !Enum.IsDefined(format.Notation) || !Enum.IsDefined(format.Padding) || !Enum.IsDefined(format.Sign))
            throw new ArgumentOutOfRangeException(nameof(format));
    }

    private static bool InstrumentLabelStepAdvances(double minimum, double maximum, double step)
    {
        // CUI-155 (docs/future/cui-upstream-bugs.md#cui-155): verify that native ticks advance.
        if (step == 0) return true;
        double first = Math.Ceiling(minimum / step - 1e-9) * step;
        double last = maximum + step * 1e-6;
        return double.IsFinite(first) && double.IsFinite(last) && first + step > first && last + step > last;
    }

    private static void CheckChartColor(CuiChartColor color)
    {
        if (!color.IsValid) throw new ArgumentOutOfRangeException(nameof(color));
    }

    private static void CheckChartOptions(CuiChartOptions options)
    {
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        if (!float.IsFinite(options.Height) || options.Height < 0 ||
            !float.IsFinite(options.Minimum) || !float.IsFinite(options.Maximum) ||
            options.Minimum > options.Maximum || !double.IsFinite(options.XMinimum) ||
            !double.IsFinite(options.XMaximum - options.XMinimum) || options.XMaximum < options.XMinimum)
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
                if (item.Label is not null) Utf8.Validate(item.Label, nameof(options));
                if (item.Detail is not null) Utf8.Validate(item.Detail, nameof(options));
                CheckChartColor(item.Color);
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
        if (bars && count > 1 && options.XMaximum > options.XMinimum)
        {
            double halfSlot = (options.XMaximum - options.XMinimum) / (count - 1) * 0.5;
            if (!double.IsFinite((options.XMaximum + halfSlot) - (options.XMinimum - halfSlot)))
                throw new ArgumentOutOfRangeException(nameof(options), "The X range cannot fit the bar slots.");
        }
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
