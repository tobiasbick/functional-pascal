using Fpas.Ui.Gui.Cui.Interop.Planning;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_month_calendar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int MonthCalendar(nint ui, string id, ref int month, ref int selected, NativeMonthEvent* items, int count, int firstWeekday, float cellHeight, byte** months, byte** days, int today, int flags, int focus, int* parts, out int activated);
}
