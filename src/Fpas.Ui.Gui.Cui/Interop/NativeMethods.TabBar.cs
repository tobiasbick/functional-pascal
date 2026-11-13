using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_tab_bar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TabBar(nint context, string id, [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] icons, [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels, int* badges, int count, ref int selected, int widthKind, float width, int flags, int focus, int* parts);
}
