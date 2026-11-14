using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_popover_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int PopoverBegin(nint context, string id, int placement, float width,
        float padding, float gap, int open, int enabled, out int node, out uint owner);

    [LibraryImport(Library, EntryPoint = "fpas_cui_popover_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void PopoverEnd(nint context, int node);
}
