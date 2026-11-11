using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_breadcrumb", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Breadcrumb(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels,
        int count, int enabled, int focus, int* nodes);
}
