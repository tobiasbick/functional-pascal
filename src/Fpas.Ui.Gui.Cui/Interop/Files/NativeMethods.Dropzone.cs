using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dropzone", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Dropzone(nint context, string id, string? title, string? description,
        string? accept, string? icon, float height, int compact, int enabled, int focus, out int node);

    [LibraryImport(Library, EntryPoint = "cui_dropped_file")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint DroppedFile(nint context, int index);
}
