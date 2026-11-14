using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_table_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TableBegin(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles,
        int* kinds, float* widths, int* alignments, int* sortable, int count, int heightKind,
        float height, int striped, int compact, int selectable, int enabled, ref int sort,
        ref int descending, int focusHeader, int* headers);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_row", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TableRow(nint context, string id, int selected, int enabled, int focus, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_cell", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TableCell(nint context, string? text);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_row_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TableRowEnd(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TableEnd(nint context, int node);
}
