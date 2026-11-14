using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_problems", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevProblems(nint ui, string id, NativeDevDiagnostic* diagnostics, int diagnostics_count, float height, int hide_toolbar, int width_kind, float width, int enabled, out NativeDevResult result);
}
