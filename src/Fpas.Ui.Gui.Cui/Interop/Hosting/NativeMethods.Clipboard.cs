// [Documentation](../../../../docs/ui/gui/cui.md#desktop-services)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_set_clipboard_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void WriteClipboardText(nint context, string text);

    [LibraryImport(Library, EntryPoint = "cui_get_clipboard_text")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint ReadClipboardText(nint context);
}
