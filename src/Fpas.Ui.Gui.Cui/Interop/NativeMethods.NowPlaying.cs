using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_now_playing", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int NowPlaying(nint context, string id, string title, string artist, string? artwork, int palette, uint rgba, ref float position, float duration, ref int playing, int widthKind, float width, int flags, int focus, int* parts);
}
