using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_tabs", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Tabs(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] icons,
        int count, int selected, int variant, int fullWidth, int enabled, int focus, int* nodes);
}
