using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_image_cropper", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ImageCropper(nint ui, string id, in NativeMediaSource source, ref CuiRectangle current, int widthKind, float width, int heightKind, float height, float aspect, float minimum, int flags, int focus);
}
