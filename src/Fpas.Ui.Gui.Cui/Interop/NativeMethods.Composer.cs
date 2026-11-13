using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_composer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Composer(nint ui, string? id, byte* draft, nuint capacity, string? placeholder, byte** attachments, int attachment_count, NativeModel* models, int model_count, int* selected, float* levels, int level_count, float seconds, int max_lines, int width_kind, float width, int flags, int focus, int* removed, int* nodes);
}
