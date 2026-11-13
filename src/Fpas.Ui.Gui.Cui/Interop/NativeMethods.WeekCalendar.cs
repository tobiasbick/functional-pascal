using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeWeekEvent
{
    internal byte* Title, Detail;
    internal int Day;
    internal float Start, Finish;
    internal int Tone;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_week_calendar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int WeekCalendar(nint ui, string id, int week, NativeWeekEvent* items, int count, float start, float finish, float hourHeight, int heightKind, float height, int flags, int focus, int* parts);
}
