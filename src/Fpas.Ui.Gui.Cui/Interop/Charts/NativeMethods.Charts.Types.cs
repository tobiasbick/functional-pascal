// [Documentation](../../../../docs/ui/gui/charts/README.md)
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartSeries
{
    internal byte* Name;
    internal float* Values;
    internal int Count, Palette;
    internal uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartAnnotation
{
    internal int Kind;
    internal double X, Y;
    internal byte* Label;
    internal byte* Detail;
    internal int Accent, Palette;
    internal uint Rgba;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeChartOptions
{
    internal int WidthKind;
    internal float Width, Height;
    internal int Flags;
    internal byte** Labels;
    internal int LabelCount;
    internal byte* Format;
    internal float Minimum, Maximum;
    internal NativeChartAnnotation* Annotations;
    internal int AnnotationCount;
    internal double XMinimum, XMaximum;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMeterPart
{
    internal byte* Name;
    internal float Value;
    internal int Palette;
    internal uint Rgba;
    internal float Density;
}
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMeterOptions
{
    internal float Total;
    internal byte* Label;
    internal byte* Unit;
    internal byte* Format;
    internal byte* FreeLabel;
    internal int Flags;
    internal float Height;
    internal int WidthKind;
    internal float Width;
}
