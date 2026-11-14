using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_hover_card_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int HoverCardBegin(nint context, string id, int placement, float width, float delay,
        int enabled, out int node, out uint owner);

    [LibraryImport(Library, EntryPoint = "fpas_cui_hover_card_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void HoverCardEnd(nint context, int anchor);
}
