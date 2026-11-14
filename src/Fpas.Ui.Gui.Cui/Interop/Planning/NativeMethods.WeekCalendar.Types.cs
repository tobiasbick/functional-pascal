using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop.Planning;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeWeekEvent
{
    internal byte* Title, Detail;
    internal int Day;
    internal float Start, Finish;
    internal int Tone;
}
