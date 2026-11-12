using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_emoji_picker", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial nint EmojiPicker(nint context, string id, byte* recent, nuint capacity,
        ref int tone, float width, float height, float size, string? placeholder, string? emptyText, string? hint,
        int search, int preview, int toneSelector, int enabled, int focus, int* nodes);
}
