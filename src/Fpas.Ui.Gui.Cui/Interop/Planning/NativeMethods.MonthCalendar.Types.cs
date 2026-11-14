using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Planning;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeMonthEvent
{
    internal byte* Title;
    internal int Start, Finish;
    internal float Time;
    internal int AllDay, Tone;
}
