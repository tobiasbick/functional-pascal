using Fpas.Ui.Gui.Cui.Widgets.Charts;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

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
