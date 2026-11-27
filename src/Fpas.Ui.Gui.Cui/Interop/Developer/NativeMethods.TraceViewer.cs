// [Documentation](../../../../docs/ui/gui/developer-tools/trace-viewer.md)
using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_trace_sort")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void TraceSort(NativeDevSpan* spans, int count);

    [LibraryImport(Library, EntryPoint = "fpas_cui_trace_viewer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevTraceViewer(nint ui, string id, NativeDevTrack* tracks, int tracks_count, NativeDevSpan* spans, int spans_count, NativeDevMarker* markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int enabled, out NativeDevResult result, NativeTraceLink* links, int linkCount, out int hoveredLink);

    [LibraryImport(Library, EntryPoint = "fpas_cui_trace_span_rect", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiRectangle TraceSpanRectangle(nint ui, string id, double start, double end, int track, int depth);
}
