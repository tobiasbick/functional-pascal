using Fpas.Ui.Gui.Cui.Widgets.Planning;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int KanbanBegin(nint ui, string id, int enabled);
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_column_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int KanbanColumnBegin(nint ui, string title, int count, int palette, uint color,
        int limit, float width, int* parts);
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_card_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int KanbanCardBegin(nint ui, int key, int focus, out int node);
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_card_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void KanbanCardEnd(nint ui);
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_column_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void KanbanColumnEnd(nint ui);
    [LibraryImport(Library, EntryPoint = "fpas_cui_kanban_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int KanbanEnd(nint ui, int node, out CuiKanbanMove move);
}
