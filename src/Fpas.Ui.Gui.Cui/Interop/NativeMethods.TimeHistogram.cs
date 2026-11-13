using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_time_histogram", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevTimeHistogram(nint ui, string id, double start, double bucket, int buckets, NativeDevSeries* series, int series_count, double range_start, double range_end, float height, int utc, int width_kind, float width, int enabled, out NativeDevResult result);
}
