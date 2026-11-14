using Fpas.Ui.Gui.Cui.Interop.Planning;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_week_calendar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int WeekCalendar(nint ui, string id, int week, NativeWeekEvent* items, int count, float start, float finish, float hourHeight, int heightKind, float height, int flags, int focus, int* parts);
}
