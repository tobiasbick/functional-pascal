using System.Globalization;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

    internal byte* Format(CuiChartFormat? format) => format is null ? null :
        Text(format.Prefix.Replace("%", "%%", StringComparison.Ordinal) + "%." +
            format.DecimalPlaces.ToString(CultureInfo.InvariantCulture) + "f" +
            format.Suffix.Replace("%", "%%", StringComparison.Ordinal));

    internal NativeChartOptions Options(CuiChartOptions options)
    {
        var annotations = options.Annotations ?? [];
        var native = new NativeChartAnnotation[annotations.Length];
        for (int i = 0; i < native.Length; i++)
        {
            var item = annotations[i];
            native[i] = new() { Kind = item.Kind, X = item.X, Y = item.Y,
                Label = Text(item.Label), Detail = Text(item.Detail), Accent = item.Accent ? 1 : 0 };
        }
        return new() { WidthKind = options.WidthKind, Width = options.Width, Height = options.Height,
            Flags = (options.Area ? 1 : 0) | (!options.Grid ? 2 : 0) | (!options.Legend ? 4 : 0) |
                (options.Stacked ? 8 : 0) | (options.Smooth ? 16 : 0) | (options.Dither ? 32 : 0),
            Labels = Labels(options.Labels), LabelCount = options.Labels?.Length ?? 0,
            Format = Format(options.Format), Minimum = options.Minimum, Maximum = options.Maximum,
            Annotations = Copy<NativeChartAnnotation>(native), AnnotationCount = native.Length };
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
