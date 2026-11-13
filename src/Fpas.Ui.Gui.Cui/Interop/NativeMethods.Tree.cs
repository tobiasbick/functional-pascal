using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TreeBegin(nint context, string id, string label, string? icon,
        int leaf, int selected, int enabled, int focus, ref int open, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tree_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TreeEnd(nint context, int node);
}
