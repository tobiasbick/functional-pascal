using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_trace_viewer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevTraceViewer(nint ui, string id, NativeDevTrack* tracks, int tracks_count, NativeDevSpan* spans, int spans_count, NativeDevMarker* markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int enabled, out NativeDevResult result);
}
