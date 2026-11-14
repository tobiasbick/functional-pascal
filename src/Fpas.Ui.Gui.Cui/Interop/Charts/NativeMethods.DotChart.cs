using Fpas.Ui.Gui.Cui.Interop.Charts;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dot_chart", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DotChart(nint context, string id, float* samples, int* accent, int series, int count,
        in NativeDotOptions options, out int index, out double x, out float value);
}
