using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_blame", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevBlame(nint ui, string id, byte** lines, int lines_count, int* line_commits, int line_commits_count, NativeDevRevision* revisions, int revisions_count, float height, string? language, long now, int first_line, int width_kind, float width, int enabled, out NativeDevResult result);
}
