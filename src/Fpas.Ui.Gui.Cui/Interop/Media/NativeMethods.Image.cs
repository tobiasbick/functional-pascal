using Fpas.Ui.Gui.Cui.Interop.Media;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_display_image", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DisplayImage(nint context, string id, in NativeMediaSource source, int widthKind, float width, int heightKind, float height, float radius, uint tint);
}
