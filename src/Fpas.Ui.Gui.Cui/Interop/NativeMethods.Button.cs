using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_button_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Button(nint context, string label, nint options);

    [LibraryImport(Library, EntryPoint = "fpas_cui_button", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int HostButton(nint context, string id, string label, int variant, int size, int enabled,
        int loading, int selected, int focus, string? icon, string? iconRight, string? tooltip, string? shortcut,
        int fill, float width, uint rgba);
}
