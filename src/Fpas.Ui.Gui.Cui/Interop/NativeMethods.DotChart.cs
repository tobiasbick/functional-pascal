using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeDotOptions
{
    internal NativeChartOptions Chart;
    internal double Minimum, Maximum, Step;
    internal byte* XFormat;
    internal byte* Unit;
    internal float Pitch;
    internal int Units, Flags;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dot_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DotChart(nint context, string id, float* samples, int* accent, int series, int count,
        in NativeDotOptions options, out int index, out double x, out float value);
}
