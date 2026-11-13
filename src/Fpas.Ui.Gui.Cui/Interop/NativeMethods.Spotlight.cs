using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_spotlight_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int SpotlightBegin(nint context, string id, int widthKind, float width, int heightKind, float height, float padding, float gap, int palette, uint rgba, int beam);

    [LibraryImport(Library, EntryPoint = "fpas_cui_spotlight_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SpotlightEnd(nint context, int node);
}
