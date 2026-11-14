using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_skeleton_group_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int SkeletonGroupBegin(nint ui, string id, int widthKind, float width, float gap, int dither);

    [LibraryImport(Library, EntryPoint = "fpas_cui_skeleton_group_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SkeletonGroupEnd(nint ui, int node);
}
