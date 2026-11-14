using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_file_browser", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int FileBrowser(nint ui, string id, byte* folder, int capacity,
        int widthKind, float width, int heightKind, float height, string? filter, string[] places,
        int placeCount, nint callback, nint user, int maximum, int revision, int flags, int focus,
        out nint selected, out nint picked, int* parts);
}
