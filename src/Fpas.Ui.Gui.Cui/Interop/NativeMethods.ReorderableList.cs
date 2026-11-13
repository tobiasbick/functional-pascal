using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_reorder_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ReorderBegin(nint context, string id, int* order, int count, int enabled, int* proposed);

    [LibraryImport(Library, EntryPoint = "fpas_cui_reorder_item_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ReorderItemBegin(nint context, int position, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_reorder_item_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ReorderItemEnd(nint context, int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_reorder_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ReorderEnd(nint context, int node);

}
