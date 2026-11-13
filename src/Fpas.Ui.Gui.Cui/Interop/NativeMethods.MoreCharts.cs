using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSankeyNode { public byte* Label; public int Palette; public uint Rgba; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeScatterSeries { public byte* Name; public CuiPlotPoint* Points; public float* Sizes; public int Count, Palette; public uint Rgba; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeGaugeBand { public float To; public int Tone; public byte* Name; }
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeRadialOptions
{
    public byte* Label; public byte* Unit; public byte* Format;
    public float Size, Sweep; public NativeGaugeBand* Bands; public int BandCount, Flags;
    public float Target; public int Style; public byte* Detail; public int Palette; public uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCandleOptions
{
    public int WidthKind; public float Width, Height; public byte** Labels; public byte* Format;
    public NativeChartSeries* Overlays; public int OverlayCount, Flags, UpPalette; public uint UpRgba;
    public int DownPalette; public uint DownRgba;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_sankey", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Sankey(nint context, string id, NativeSankeyNode* nodes, int count,
        CuiSankeyLink* links, int linkCount, int widthKind, float width, float height, float nodeWidth,
        byte* format, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_bar_list", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int BarList(nint context, string id, NativeVizItem* items, int count,
        int widthKind, float width, byte* format, int maximumRows, int keepOrder, int palette, uint rgba,
        int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_activity_rings", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ActivityRings(nint context, string id, NativeVizItem* items, int count,
        float size, float thickness, byte* center, byte* caption, int legend, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_scatter_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ScatterChart(nint context, string id, NativeScatterSeries* series, int count,
        int widthKind, float width, float height, byte* xLabel, byte* yLabel, byte* xFormat, byte* yFormat,
        int legend, int enabled, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_radial_gauge", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int RadialGauge(nint context, string id, float value, float minimum, float maximum,
        in NativeRadialOptions options);

    [LibraryImport(Library, EntryPoint = "fpas_cui_candlestick", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Candlestick(nint context, string id, CuiCandle* candles, int count,
        in NativeCandleOptions options, int enabled, int focus, out int node);
}
