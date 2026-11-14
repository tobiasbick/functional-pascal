using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_path_bar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int PathBar(nint ui, string id, string[] segments, int count, string? text,
        string? icon, string? placeholder, int size, int widthKind, float width, int flags, int focus,
        out nint submitted, int* parts, out int root, out uint retainedKey);
    [LibraryImport(Library, EntryPoint = "fpas_cui_path_menu_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int PathMenuBegin(nint ui, string id);
    [LibraryImport(Library, EntryPoint = "fpas_cui_path_menu_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int PathMenuItem(nint ui, string label, int index);
    [LibraryImport(Library, EntryPoint = "fpas_cui_path_menu_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void PathMenuEnd(nint ui);
}
