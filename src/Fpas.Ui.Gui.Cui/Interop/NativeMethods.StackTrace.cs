using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_stack_trace", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevStackTrace(nint ui, string id, NativeDevStackFrame* frames, int frames_count, string? title, string? message, string? language, int width_kind, float width, int enabled, out NativeDevResult result);
}
