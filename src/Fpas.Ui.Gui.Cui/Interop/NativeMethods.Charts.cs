using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartSeries { public byte* Name; public float* Values; public int Count; public int Palette; public uint Rgba; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartAnnotation { public int Kind; public double X, Y; public byte* Label; public byte* Detail; public int Accent; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartOptions
{
    public int WidthKind; public float Width, Height; public int Flags;
    public byte** Labels; public int LabelCount; public byte* Format;
    public float Minimum, Maximum; public NativeChartAnnotation* Annotations; public int AnnotationCount;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMeterPart { public byte* Name; public float Value; public int Palette; public uint Rgba; public float Density; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMeterOptions
{
    public float Total; public byte* Label; public byte* Unit; public byte* Format; public byte* FreeLabel;
    public int Flags; public float Height; public int WidthKind; public float Width;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_line_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int LineChart(nint context, string id, NativeChartSeries* series, int count, in NativeChartOptions options);
    [LibraryImport(Library, EntryPoint = "fpas_cui_bar_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int BarChart(nint context, string id, NativeChartSeries* series, int count, in NativeChartOptions options);
    [LibraryImport(Library, EntryPoint = "fpas_cui_donut_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DonutChart(nint context, string id, float* values, int count, in NativeChartOptions options);
    [LibraryImport(Library, EntryPoint = "fpas_cui_gauge", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Gauge(nint context, string id, string label, float value, float size);
    [LibraryImport(Library, EntryPoint = "fpas_cui_pattern_meter", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int PatternMeter(nint context, string id, NativeMeterPart* parts, int count,
        in NativeMeterOptions options, int enabled, int focus, out int node, out uint key);
}
