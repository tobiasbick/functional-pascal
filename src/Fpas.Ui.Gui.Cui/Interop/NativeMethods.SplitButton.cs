using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_split_button", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int SplitButton(nint context, string id, string label, string? icon,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] icons,
        int count, int variant, int size, int enabled, int focus, int* nodes, out int open, out int highlighted);
}
