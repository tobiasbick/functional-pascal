using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_checklist", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Checklist(nint context, string id, string? title, string? description,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] descriptions,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] actions,
        int* completed, int count, int enabled, int focus, out int node, int* parts);
}
