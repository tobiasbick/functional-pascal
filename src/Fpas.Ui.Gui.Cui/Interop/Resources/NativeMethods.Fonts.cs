using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_font_load")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int FontLoad(nint ui, byte* bytes, nuint length);
    [LibraryImport(Library, EntryPoint = "fpas_cui_font_role")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int FontRole(nint ui, int role);
    [LibraryImport(Library, EntryPoint = "fpas_cui_font_set_role")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void FontSetRole(nint ui, int role, int font);
    [LibraryImport(Library, EntryPoint = "fpas_cui_font_fallback")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void FontFallback(nint ui, int font, int fallback);
    [LibraryImport(Library, EntryPoint = "fpas_cui_emoji_load")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int EmojiLoad(nint ui, byte* bytes, nuint length);
    [LibraryImport(Library, EntryPoint = "fpas_cui_emoji_system")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int EmojiSystem(nint ui);
    [LibraryImport(Library, EntryPoint = "fpas_cui_emoji_supported", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int EmojiSupported(nint ui, string text);
}
