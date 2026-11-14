using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Charts;

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
