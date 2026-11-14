using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_histogram_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int HistogramSlider(nint context, string id, string label, float* bins,
        int binCount, ref float lower, ref float upper, float minimum, float maximum, float height,
        string format, int enabled, int focus, out int track);
}
