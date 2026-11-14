using Fpas.Ui.Gui.Cui.Interop.Charts;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

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
