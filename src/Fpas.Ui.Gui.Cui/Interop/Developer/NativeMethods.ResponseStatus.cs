// [Documentation](../../../../docs/ui/gui/more-developer-tools/http-response-status.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_response_status", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DevResponseStatus(nint ui, string id, int status, double seconds, double bytes, int widthKind, float width, int enabled);
}
