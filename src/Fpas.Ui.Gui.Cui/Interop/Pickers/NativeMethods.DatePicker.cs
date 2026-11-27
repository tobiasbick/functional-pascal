// [Documentation](../../../../docs/ui/gui/inputs/date-picker.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_date_picker", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DatePicker(nint context, string id, int value, string? label,
        string? placeholder, int widthKind, float width, int enabled, int focus, out int trigger, int* parts, int* nodes,
        out int count);
}
