// [Documentation](../../../../docs/ui/gui/feedback/status-spinner.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_status_spinner", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int StatusSpinner(nint ui, string id, int status, float size, int themeToken, uint rgba, int keep);
}
