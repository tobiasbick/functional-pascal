using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeVizItem { public byte* Label; public float Value; public int Palette; public uint Rgba; }

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_heatmap", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Heatmap(nint context, string id, float* values, int rows, int columns,
        byte** rowLabels, byte** columnLabels, float cell, float minimum, float maximum, int palette, uint rgba,
        byte* format, int widthKind, float width, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_calendar_heatmap", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int CalendarHeatmap(nint context, string id, int year, int month, int day,
        float* values, int count, float cell, int palette, uint rgba, byte* format, byte* unit, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_treemap", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Treemap(nint context, string id, NativeVizItem* items, int count,
        int widthKind, float width, float height, byte* format, int selectable, ref int selected, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_radar_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int RadarChart(nint context, string id, NativeChartSeries* series, int count,
        byte** axes, int axisCount, float size, float maximum, byte* format, int legend, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_funnel", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Funnel(nint context, string id, NativeVizItem* items, int count,
        int widthKind, float width, int palette, uint rgba, byte* format, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_split_bar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int SplitBar(nint context, string id, NativeVizItem* items, int count,
        int widthKind, float width, float height, byte* format, int legend, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_waffle", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Waffle(nint context, string id, NativeVizItem* items, int count,
        float cell, int columns, int rows, byte* format, int enabled, out int node);
}
