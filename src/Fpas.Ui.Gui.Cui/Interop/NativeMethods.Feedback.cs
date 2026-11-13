using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_alert", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Alert(nint context, string id, string title, string description,
        string icon, int tone, int dismissible, int enabled, int focus,
        out int node, out int dismiss, out uint key);

    [LibraryImport(Library, EntryPoint = "fpas_cui_progress", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Progress(nint context, string id, float value, int tone,
        int indeterminate, float height, string label, int showValue, int dither);

    [LibraryImport(Library, EntryPoint = "fpas_cui_spinner", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Spinner(nint context, string id, float size);

    [LibraryImport(Library, EntryPoint = "fpas_cui_meter", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Meter(nint context, string id, string label, float used, float total, string unit);

    [LibraryImport(Library, EntryPoint = "fpas_cui_progress_ring", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ProgressRing(nint context, string id, string label, float value, float size);

    [LibraryImport(Library, EntryPoint = "fpas_cui_timeline_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TimelineItem(nint context, string id, string title, string time,
        string description, string icon, int tone, int last);

    [LibraryImport(Library, EntryPoint = "fpas_cui_skeleton", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Skeleton(nint context, string id, int widthKind, float width, float height);
}
