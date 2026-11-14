using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_carousel_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int CarouselBegin(nint ui, string id, int count, int index, float height, float gap, float advance, int flags, int focus);

    [LibraryImport(Library, EntryPoint = "fpas_cui_carousel_slide_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CarouselSlideBegin(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_carousel_slide_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CarouselSlideEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_carousel_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int CarouselEnd(nint ui, int root, int enabled, int* parts);
}
