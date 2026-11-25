// [Documentation](../../../../docs/ui/gui/more-charts/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeSankeyNode
{
    internal byte* Label;
    internal int Palette;
    internal uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeScatterSeries
{
    internal byte* Name;
    internal CuiPlotPoint* Points;
    internal float* Sizes;
    internal int Count, Palette;
    internal uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeGaugeBand
{
    internal float To;
    internal int Tone;
    internal byte* Name;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeRadialOptions
{
    internal byte* Label;
    internal byte* Unit;
    internal byte* Format;
    internal float Size, Sweep;
    internal NativeGaugeBand* Bands;
    internal int BandCount, Flags;
    internal float Target;
    internal int Style;
    internal byte* Detail;
    internal int Palette;
    internal uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCandleOptions
{
    internal int WidthKind;
    internal float Width, Height;
    internal byte** Labels;
    internal byte* Format;
    internal NativeChartSeries* Overlays;
    internal int OverlayCount, Flags, UpPalette;
    internal uint UpRgba;
    internal int DownPalette;
    internal uint DownRgba;
}
