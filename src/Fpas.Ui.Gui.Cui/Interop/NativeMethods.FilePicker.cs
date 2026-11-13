using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_file_picker", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int FilePicker(nint ui, string id, int open, int enabled, int mode,
        string? title, string? start, string? name, string? filter, string? filterLabel, nint callback,
        nint user, byte* path, int capacity, int* parts);
}
