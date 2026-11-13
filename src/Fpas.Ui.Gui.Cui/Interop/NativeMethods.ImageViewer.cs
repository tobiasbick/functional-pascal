using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_image_viewer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ImageViewer(nint ui, string id, in NativeMediaSource source, ref CuiImageView current, int widthKind, float width, int heightKind, float height, float maximum, int flags, int focus);
}
