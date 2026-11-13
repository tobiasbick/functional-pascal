using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeGanttTask
{
    internal byte* Name, Owner;
    internal int Start, Finish;
    internal float Progress;
    internal int Tone, Dependency, Milestone, State;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_gantt", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Gantt(nint ui, string id, int origin, int days, NativeGanttTask* tasks, int count, int today, float dayWidth, float rowHeight, int heightKind, float height, int flags, int focus, int* parts);
}
