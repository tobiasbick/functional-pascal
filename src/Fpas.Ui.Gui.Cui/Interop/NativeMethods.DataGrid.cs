using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_data_grid_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial nint DataGridBegin(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles,
        float* widths, int* flags, int columns, int rows, byte* selected, int* state,
        int heightKind, float height, float rowHeight, int options, int* range, int* headers);

    [LibraryImport(Library, EntryPoint = "fpas_cui_data_grid_row", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DataGridRow(nint context, int row, string id);

    [LibraryImport(Library, EntryPoint = "fpas_cui_data_grid_cell", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DataGridCell(nint context, string? text);

    [LibraryImport(Library, EntryPoint = "fpas_cui_data_grid_edit")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DataGridEdit(nint context, byte* value, int capacity);

    [LibraryImport(Library, EntryPoint = "fpas_cui_data_grid_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DataGridEnd(nint context, nint handle, byte* selected, int* state);
}
