// [Documentation](../../../../docs/ui/gui/inputs/text-area.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_text_area", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TextArea(nint context, string id, byte* value, nuint capacity,
        string? label, string? placeholder, string? icon, string? hint, string? error, string? prefix, string? suffix,
        int size, float height, int clearable, int readOnly, int enabled, int focus,
        int widthKind, float width, int autofocus, int* nodes);
}
