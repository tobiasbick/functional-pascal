using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_shortcut_owners")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ShortcutOwners(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_take_shortcut")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TakeShortcut(nint context, int modifiers, int key);

    [LibraryImport(Library, EntryPoint = "fpas_cui_shortcut_keys")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ShortcutKeys(nint context, byte* keys, int count);
}
