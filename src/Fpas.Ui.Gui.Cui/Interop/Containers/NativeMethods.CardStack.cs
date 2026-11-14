using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_card_stack_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int CardStackBegin(nint ui, string id, int count, int widthKind, float width, float height, int visible, int enabled, int focus);

    [LibraryImport(Library, EntryPoint = "fpas_cui_card_stack_item_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CardStackItemBegin(nint ui, int depth);

    [LibraryImport(Library, EntryPoint = "fpas_cui_card_stack_item_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CardStackItemEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_card_stack_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int CardStackEnd(nint ui, int root, int enabled, out int top);
}
