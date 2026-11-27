// [Documentation](../../../../docs/ui/gui/data-grid/tree-table.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [StructLayout(LayoutKind.Sequential)]
    internal record struct NativeTreeNode(int Parent, int FirstChild, int NextSibling, uint Flags);

    [LibraryImport(Library, EntryPoint = "cui_tree_move")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TreeMove(NativeTreeNode* nodes, int count, int root,
        int* moved, int movedCount, int target, int placement);

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
        float rowHeight, int widthKind, float width, int options, byte* dropPermissions, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_row")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableRow(nint context, out int part);

    [LibraryImport(Library, EntryPoint = "cui_tree_table_depth")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableDepth(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_name", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableName(nint context, string label, string? icon);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void TreeTableEnd(nint context, nint handle, uint* flags, int* state, int* events, int* load, int* dropped);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_menu_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeTableMenuBegin(nint context, string id, int enabled, out int node, out uint popup, out int table);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_table_menu_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TreeTableMenuEnd(nint context, uint popup, int table, int visible,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] keys,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] icons,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] shortcuts,
        NativeMenuRow* rows, int count, int* nodes, out int open, out int highlighted, out uint owner);
}
