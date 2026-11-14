using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dialog_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DialogBegin(nint context, string id, string title, string? description,
        float width, int open, int enabled, int persistent, int closeButton, int focus, out int panel);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dialog_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DialogEnd(nint context, int baseNode);
}
