using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_shortcut_recorder", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ShortcutRecorder(nint context, string id, ref int key, ref int mods,
        string? placeholder, int widthKind, float width, int enabled, int focus, int* nodes);

    [LibraryImport(Library, EntryPoint = "fpas_cui_shortcut_label")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ShortcutLabel(nint context, int key, int mods, byte* value, nuint capacity);

    [LibraryImport(Library, EntryPoint = "fpas_cui_shortcut_display")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ShortcutDisplay(nint context, int key, int mods, byte* value, nuint capacity);
}
