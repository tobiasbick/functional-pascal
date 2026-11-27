// [Documentation](../../../../docs/ui/gui/charts/README.md)
using Fpas.Ui.Gui.Cui.Interop.Charts;
using System.Globalization;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Charts;

// Call-scoped storage. Native CUI copies all data needed by end-of-frame painting.
internal sealed unsafe partial class ChartMemory : IDisposable
{
    private readonly List<nint> allocations = [];

    internal T* Copy<T>(ReadOnlySpan<T> values) where T : unmanaged
    {
        if (values.IsEmpty) return null;
        T* pointer = (T*)NativeMemory.Alloc(checked((nuint)values.Length * (nuint)sizeof(T)));
        allocations.Add((nint)pointer);
        values.CopyTo(new Span<T>(pointer, values.Length));
        return pointer;
    }

    internal byte* Text(string? value)
    {
        if (value is null) return null;
        return Copy<byte>(Utf8.Encoding.GetBytes(value + '\0'));
    }

    internal byte** Labels(string[]? labels)
    {
        if (labels is null || labels.Length == 0) return null;
        nint[] pointers = new nint[labels.Length];
        for (int i = 0; i < labels.Length; i++) pointers[i] = (nint)Text(labels[i]);
        return (byte**)Copy<nint>(pointers);
    }

    internal byte* Format(CuiChartFormat? format) => Text(FormatString(format));

    internal static string? FormatString(CuiChartFormat? format)
    {
        if (format is null) return null;
        string padding = format.Padding switch
        {
            CuiValuePadding.TrailingSpaces => "-",
            CuiValuePadding.LeadingZeros => "0",
            _ => ""
        };
        string sign = format.Sign switch
        {
            CuiValueSign.Always => "+",
            CuiValueSign.Space => " ",
            _ => ""
        };
        char conversion = format.Notation switch
        {
            CuiValueNotation.Scientific => 'e',
            CuiValueNotation.General => 'g',
            CuiValueNotation.Hexadecimal => 'a',
            _ => 'f'
        };
        if (format.Uppercase) conversion = char.ToUpperInvariant(conversion);
        string width = format.FieldWidth > 0 ? format.FieldWidth.ToString(CultureInfo.InvariantCulture) : "";
        string precision = format.DecimalPlaces is int digits ? "." + digits.ToString(CultureInfo.InvariantCulture) : "";
        string prefix = format.Prefix.Replace("%", "%%", StringComparison.Ordinal);
        string suffix = format.Suffix.Replace("%", "%%", StringComparison.Ordinal);
        return $"{prefix}%{padding}{sign}{(format.Alternate ? "#" : "")}{width}{precision}{conversion}{suffix}";
    }

    internal NativeChartOptions Options(CuiChartOptions options)
    {
        var annotations = options.Annotations ?? [];
        var native = new NativeChartAnnotation[annotations.Length];
        for (int i = 0; i < native.Length; i++)
        {
            var item = annotations[i];
            native[i] = new() { Kind = item.Kind, X = item.X, Y = item.Y,
                Label = Text(item.Label), Detail = Text(item.Detail), Accent = item.Accent ? 1 : 0,
                Palette = item.Color.Palette, Rgba = item.Color.Rgba };
        }
        return new() { WidthKind = options.WidthKind, Width = options.Width, Height = options.Height,
            Flags = (options.Area ? 1 : 0) | (!options.Grid ? 2 : 0) | (!options.Legend ? 4 : 0) |
                (options.Stacked ? 8 : 0) | (options.Smooth ? 16 : 0) | (options.Dither ? 32 : 0),
            Labels = Labels(options.Labels), LabelCount = options.Labels?.Length ?? 0,
            Format = Format(options.Format), Minimum = options.Minimum, Maximum = options.Maximum,
            Annotations = Copy<NativeChartAnnotation>(native), AnnotationCount = native.Length,
            XMinimum = options.XMinimum, XMaximum = options.XMaximum };
    }

    internal NativeChartSeries* Series(CuiChartSeries[] series)
    {
        var native = new NativeChartSeries[series.Length];
        for (int i = 0; i < native.Length; i++)
            native[i] = new() { Name = Text(series[i].Name), Values = Copy<float>(series[i].Values),
                Count = series[i].Values.Length, Palette = series[i].Color.Palette, Rgba = series[i].Color.Rgba };
        return Copy<NativeChartSeries>(native);
    }

    internal NativeVizItem* Items(CuiVizItem[] items)
    {
        var native = new NativeVizItem[items.Length];
        for (int i = 0; i < native.Length; i++)
            native[i] = new() { Label = Text(items[i].Label), Value = items[i].Value,
                Palette = items[i].Color.Palette, Rgba = items[i].Color.Rgba };
        return Copy<NativeVizItem>(native);
    }

    internal NativeMeterPart* Parts(CuiMeterPart[] parts)
    {
        var native = new NativeMeterPart[parts.Length];
        for (int i = 0; i < native.Length; i++)
            native[i] = new() { Name = Text(parts[i].Name), Value = parts[i].Value,
                Palette = parts[i].Color.Palette, Rgba = parts[i].Color.Rgba, Density = parts[i].Density };
        return Copy<NativeMeterPart>(native);
    }

    internal NativeMeterOptions Options(CuiMeterOptions options) => new() {
        Total = options.Total, Label = Text(options.Label), Unit = Text(options.Unit),
        Format = Format(options.Format), FreeLabel = Text(options.FreeLabel),
        Flags = (options.Grayscale ? 1 : 0) | (!options.Legend ? 2 : 0), Height = options.Height,
        WidthKind = options.WidthKind, Width = options.Width };

    public void Dispose()
    {
        foreach (nint pointer in allocations) NativeMemory.Free((void*)pointer);
        allocations.Clear();
    }
}
