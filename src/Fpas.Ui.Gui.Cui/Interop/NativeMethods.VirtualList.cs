using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_virtual_list_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int VirtualListBegin(nint context, string id, int count, float rowHeight, int kind, float height, int enabled, out int first, out int last);

    [LibraryImport(Library, EntryPoint = "fpas_cui_virtual_list_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void VirtualListEnd(nint context, int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_virtual_row_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void VirtualRowBegin(nint context, int index);

    [LibraryImport(Library, EntryPoint = "fpas_cui_virtual_row_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void VirtualRowEnd(nint context);

}
