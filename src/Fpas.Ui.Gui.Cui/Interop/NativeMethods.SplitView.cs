using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_split_view_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial float SplitViewBegin(nint ui, string id, float ratio, int vertical, float first, float second, int widthKind, float width, int heightKind, float height, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_split_view_next", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SplitViewNext(nint ui, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_split_view_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SplitViewEnd(nint ui);
}
