using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_range_calendar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int RangeCalendar(nint context, string id, ref int start, ref int end,
        int months, int enabled, out int root, int* parts, int* nodes, out int count);

    [LibraryImport(Library, EntryPoint = "fpas_cui_range_picker", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int RangePicker(nint context, string id, ref int start, ref int end,
        string? label, string? placeholder, float width, int presets, int enabled, int focus,
        out int trigger, int* parts, int* nodes, out int count, out int open);
}
