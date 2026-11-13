using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMonthEvent
{
    internal byte* Title;
    internal int Start, Finish;
    internal float Time;
    internal int AllDay, Tone;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_month_calendar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int MonthCalendar(nint ui, string id, ref int month, ref int selected, NativeMonthEvent* items, int count, int firstWeekday, float cellHeight, byte** months, byte** days, int today, int flags, int focus, int* parts, out int activated);
}
