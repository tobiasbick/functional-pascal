using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dock", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Dock(nint context, string id, [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] icons, [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] labels, int* palettes, uint* colors, int* badges, int* kinds, int count, float size, float magnify, int enabled, int focus, int* parts);
}
