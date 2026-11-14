using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private static void CheckVizDistance(float value)
    {
        if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(nameof(value));
    }

    private static void CheckVizLabels(string[]? labels, int count)
    {
        if (labels is null) return;
        if (labels.Length != 0 && labels.Length != count)
            throw new ArgumentException("Labels must be empty or match the corresponding dimension.", nameof(labels));
        foreach (string label in labels) Utf8.Validate(label, nameof(labels));
    }

    private static void CheckVizSamples(float[] values, bool nonnegative)
    {
        ArgumentNullException.ThrowIfNull(values);
        foreach (float value in values)
            if (!float.IsFinite(value) || nonnegative && value < 0) throw new ArgumentOutOfRangeException(nameof(values));
    }

    private static void CheckVizItems(CuiVizItem[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        float total = 0;
        foreach (var item in items)
        {
            ArgumentNullException.ThrowIfNull(item); Utf8.Validate(item.Label, nameof(items));
            CheckChartColor(item.Color); CheckVizDistance(item.Value); total += item.Value;
        }
        if (!float.IsFinite(total)) throw new ArgumentOutOfRangeException(nameof(items));
    }

    private static void CheckHeatmap(float[] values, int rows, int columns, CuiHeatmapOptions options)
    {
        if (rows < 0 || columns < 0 || (long)rows * columns != values.Length)
            throw new ArgumentOutOfRangeException(nameof(rows), "The matrix must match rows times columns.");
        CheckVizSamples(values, false); CheckVizLabels(options.RowLabels, rows);
        CheckVizLabels(options.ColumnLabels, columns); CheckVizDistance(options.Cell);
        CheckExtent(options.WidthKind, options.Width, nameof(options)); CheckChartColor(options.Color);
        CheckChartFormat(options.Format);
        float lo = options.Minimum, hi = options.Maximum;
        if (!float.IsFinite(lo) || !float.IsFinite(hi) || lo > hi) throw new ArgumentOutOfRangeException(nameof(options));
        if (lo == hi)
        {
            lo = 1e30f; hi = -1e30f;
            foreach (float value in values) { lo = Math.Min(lo, value); hi = Math.Max(hi, value); }
            if (values.Length == 0) { lo = 0; hi = 1; }
            if (hi <= lo) hi = lo + 1;
        }
        // CUI-019 (docs/future/cui-upstream-bugs.md#cui-019): a one-unit expansion can collapse.
        if (hi <= lo || !float.IsFinite(hi - lo))
            throw new ArgumentOutOfRangeException(nameof(options), "Supply wider explicit heatmap bounds.");
    }

    private static void CheckRadar(CuiChartSeries[] series, string[] axes, CuiRadarOptions options)
    {
        ArgumentNullException.ThrowIfNull(series); ArgumentNullException.ThrowIfNull(axes);
        if (axes.Length < 3) throw new ArgumentOutOfRangeException(nameof(axes), "A radar chart needs at least three axes.");
        foreach (string label in axes) Utf8.Validate(label, nameof(axes));
        CheckVizDistance(options.Size); CheckVizDistance(options.Maximum); CheckChartFormat(options.Format);
        float maximum = 0;
        foreach (var item in series)
        {
            ArgumentNullException.ThrowIfNull(item); Utf8.Validate(item.Name, nameof(series));
            CheckChartColor(item.Color); CheckVizSamples(item.Values, true);
            if (item.Values.Length != axes.Length)
                throw new ArgumentException("Each radar series must match the axes.", nameof(series));
            foreach (float value in item.Values) maximum = Math.Max(maximum, value);
        }
        float high = options.Maximum > 0 ? options.Maximum : maximum > 0 ? maximum : 1;
        float step = options.Maximum > 0 ? high / 4 : MathF.Pow(10, MathF.Floor(MathF.Log10(high / 4)));
        if (step <= 0 || !float.IsFinite(step) || !float.IsFinite(MathF.Ceiling(high / step) * step))
            throw new ArgumentOutOfRangeException(nameof(options), "The native radar cannot represent this scale.");
    }
}
