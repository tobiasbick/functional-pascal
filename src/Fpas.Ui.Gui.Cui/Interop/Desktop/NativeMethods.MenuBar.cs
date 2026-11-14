using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_menu_bar_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MenuBarBegin(nint ui, string id, int border, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_menu_bar_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MenuBarEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_menu_bar_menu", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int MenuBarMenu(nint ui, string title, int mnemonic, float width, int enabled, string[] keys, string[] labels, string[] icons, string[] shortcuts, int* flags, int count, int* nodes, out int open, out int highlighted);
}
