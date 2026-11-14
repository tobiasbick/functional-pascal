using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_cell", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeCell(nint context, string id, string label, string? icon, int depth, ref int open, int enabled, out int part);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial nint TreeTableBegin(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles,
        float* widths, int* columnFlags, int columns,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels,
        int* parents, uint* flags, int count, int* state, int heightKind, float height,
        float rowHeight, int options, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_row")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableRow(nint context, out int part);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_name", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableName(nint context, string label, string? icon);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void TreeTableEnd(nint context, nint handle, uint* flags, int* state, int* events, int* load);
}
