using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Planning;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeGanttTask
{
    internal byte* Name, Owner;
    internal int Start, Finish;
    internal float Progress;
    internal int Tone, Dependency, Milestone, State;
}
