using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_image_compare", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ImageCompare(nint ui, string id, in NativeMediaSource before, in NativeMediaSource after, ref float position, int widthKind, float width, float height, float aspect, string beforeLabel, string afterLabel, int enabled, int focus);
}
